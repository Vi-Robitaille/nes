use crate::consts::{
    emulation_consts::*,
    emulation_consts::{CLIENT_FORMAT, COLOR_CHANNELS},
    ppu_consts::SPR_PATTERN_TABLE_SIZE,
    render_consts::*,
};

use crate::Nes;

use glium::{
    backend::Facade,
    texture::RawImage2d,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Texture2d,
};
use imgui::*;
use imgui_glium_renderer::Texture;
use std::borrow::Cow;
use std::rc::Rc;
use std::time::Instant;

pub trait EmulatedDevice {
    fn reset(&mut self);
    fn clock(&mut self);
}

pub trait BusDevice {
    fn read_one_byte(&mut self, addr: u16, ro: bool) -> u8;
    fn write_byte(&mut self, addr: u16, data: u8, ro: bool);
}

pub trait DisplayDevice {
    fn display_read_one_byte(&mut self, addr: u16, ro: bool) -> u8;
    fn display_write_byte(&mut self, addr: u16, data: u8, ro: bool);
}

pub struct EmulationControls {}

pub struct EmulationState {
    pub nes_texture_id: Option<TextureId>,
    pub frame_sync: FrameSync,
    pub palette_id: u8,
    pub debug_textures: Option<DebugTextures>,
    pub last_frame_time: Instant,
    pub cycles: usize,
    pub watch_addr: u16,
}

impl EmulationState {
    pub fn new() -> EmulationState {
        EmulationState {
            nes_texture_id: None,
            frame_sync: EMU_START_STATE,
            palette_id: 0,
            debug_textures: None,
            last_frame_time: Instant::now(),
            cycles: 0,
            watch_addr: 0x0000,
        }
    }

    fn reset(&mut self, nes: &mut Nes) {
        self.cycles = 0;
        self.watch_addr = 0;
        self.frame_sync = FrameSync::Stop;
        nes.reset();
    }

    pub fn run<F>(&mut self, nes: &mut Nes, gl_ctx: &F, tex: &mut Textures<Texture>)
    where
        F: Facade,
    {
        match self.frame_sync {
            FrameSync::Run => {
                nes.clock_one_frame();
            }
            FrameSync::OneFrame => {
                nes.clock_one_frame();
                self.frame_sync = FrameSync::Stop;
            }
            FrameSync::OneCycle => {
                nes.clock();
            }
            FrameSync::StepOneCycle => {
                nes.clock();
                self.frame_sync = FrameSync::Stop;
            }
            FrameSync::OneInstruction => {
                nes.clock_one_instruction();
                self.frame_sync = FrameSync::Stop;
            }
            FrameSync::OneScanline => {
                nes.clock_one_scanline();
                self.frame_sync = FrameSync::Stop;
            }
            FrameSync::XCycles => {
                while self.cycles > 0 {
                    nes.clock_one_instruction();
                    self.cycles -= 1;
                }
                self.frame_sync = FrameSync::Stop;
            }
            FrameSync::PCWatch => {
                println!("Running to instruction {:04X}", self.watch_addr);
                if self.watch_addr != 0 {
                    while self.watch_addr != nes.cpu.pc {
                        nes.clock_one_instruction();
                    }
                }
                self.frame_sync = FrameSync::Stop;
            }
            FrameSync::Reset => {
                self.reset(nes);
            }
            FrameSync::Stop => {}
            _ => self.frame_sync = FrameSync::Stop, /* the rest are to be implemented */
        }
        if let Some(tex_id) = self.nes_texture_id {
            let _ = self.update_display(nes, tex_id, gl_ctx, tex);
        }
    }

    pub fn update_display<F>(
        &self,
        nes: &mut Nes,
        texture_id: TextureId,
        gl_ctx: &F,
        textures: &mut Textures<Texture>,
    ) -> Result<(), anyhow::Error>
    where
        F: Facade,
    {
        let bytes = nes.cpu.bus.ppu.get_screen().to_vec();
        let texture = convert_data_to_texture(
            SCREEN_TEX_WIDTH,
            SCREEN_TEX_HEIGHT,
            bytes,
            gl_ctx,
        )?;
        if let Some(tex) = textures.get_mut(texture_id) {
            *tex = texture;
        }
        if cfg!(debug_assertions) {
            if let Some(debug_tex) = &self.debug_textures {
                let bytes = nes.get_pattern_table(0, self.palette_id).to_vec();
                // println!("{:?}", bytes);
                let texture = convert_data_to_texture(
                    SPR_PATTERN_TABLE_SIZE,
                    SPR_PATTERN_TABLE_SIZE,
                    bytes,
                    gl_ctx,
                )?;
                if let Some(tex) = textures.get_mut(debug_tex.palette_one) {
                    *tex = texture;
                }
    
                let bytes = nes.get_pattern_table(1, self.palette_id).to_vec();
                let texture = convert_data_to_texture(
                    SPR_PATTERN_TABLE_SIZE,
                    SPR_PATTERN_TABLE_SIZE,
                    bytes,
                    gl_ctx,
                )?;
                if let Some(tex) = textures.get_mut(debug_tex.palette_two) {
                    *tex = texture;
                }
            }
        }
        Ok(())
    }

    pub fn register_textures<F>(
        &mut self,
        gl_ctx: &F,
        textures: &mut Textures<Texture>,
    ) -> Result<(), anyhow::Error>
    where
        F: Facade,
    {
        if self.nes_texture_id.is_none() {
            // Generate dummy texture
            let texture_id = generate_dummy_texture(
                SCREEN_TEX_WIDTH,
                SCREEN_TEX_HEIGHT,
                gl_ctx,
                textures,
            )?;
            self.nes_texture_id = Some(texture_id);
        }

        if cfg!(debug_assertions) {
            self.debug_textures = Some(DebugTextures {
                palette_one: generate_dummy_texture(
                    SPR_PATTERN_TABLE_SIZE,
                    SPR_PATTERN_TABLE_SIZE,
                    gl_ctx,
                    textures,
                )?,
                palette_two: generate_dummy_texture(
                    SPR_PATTERN_TABLE_SIZE,
                    SPR_PATTERN_TABLE_SIZE,
                    gl_ctx,
                    textures,
                )?,
            });
        }

        Ok(())
    }

    pub fn increment_palette_id(&mut self) {
        let (r, _) = self.palette_id.overflowing_add(1);
        self.palette_id = r % 8;
    }

    pub fn decrement_palette_id(&mut self) {
        let (r, _) = self.palette_id.overflowing_sub(1);
        self.palette_id = r % 8;
    }
}

pub enum FrameSync {
    Run,              /* Hold thread until frame is complete */
    DisplayAvailable, /* Display whatever is available in the ppu */
    DisplayPrevious,  /* redraw the previous frame */
    RedrawAvailable,  /* Skip redraw until frame is available (?) */
    StepOneCycle,     /* Run the bus clock once, Then stop */
    OneCycle,         /* Run the bus clock once, Then draw the output */
    OneInstruction,   /* Run one instruction then stop */
    OneScanline,      /* Only draw one scanline */
    OneFrame, /* Only draw one frame, Control Flow should drop thihs to Stop after the frame */
    XCycles,  /* Run for x cycles */
    PCWatch,  /* Run until the program counter hits this addr */
    Stop,     /* Do nothing */
    Reset,
}

pub struct DebugTextures {
    pub palette_one: TextureId,
    pub palette_two: TextureId,
}

fn generate_dummy_texture<F>(
    w: usize,
    h: usize,
    gl_ctx: &F,
    textures: &mut Textures<Texture>,
) -> Result<TextureId, anyhow::Error>
where
    F: Facade,
{
    // Generate dummy texture
    let mut data = Vec::with_capacity(w * h * COLOR_CHANNELS);
    for i in 0..h {
        for j in 0..w {
            // Insert RGB values
            data.push(i as u8);
            data.push(j as u8);
            data.push((i + j) as u8);
        }
    }
    let texture = convert_data_to_texture(w, h, data, gl_ctx)?;
    let texture_id = textures.insert(texture);
    Ok(texture_id)
}

fn convert_data_to_texture<F>(
    w: usize,
    h: usize,
    bytes: Vec<u8>,
    gl_ctx: &F,
) -> Result<Texture, anyhow::Error>
where
    F: Facade,
{
    let raw = RawImage2d {
        data: Cow::Owned(bytes),
        width: w as u32,
        height: h as u32,
        format: CLIENT_FORMAT,
    };
    let gl_texture = Texture2d::new(gl_ctx, raw)?;
    let texture = Texture {
        texture: Rc::new(gl_texture),
        sampler: SamplerBehavior {
            magnify_filter: MagnifySamplerFilter::Linear,
            minify_filter: MinifySamplerFilter::Linear,
            ..Default::default()
        },
    };
    Ok(texture)
}
