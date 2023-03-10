const SCALING: f32 = std::f32::consts::FRAC_2_PI;

use crate::ppu::structures::Register;

struct AudioEnvelope {
    length: u8,
    frequency: u8,
}



pub struct APU2A03 {
    
}

impl APU2A03 {
    fn cpu_read(addr: u16) -> u8 {
        0
    }

    fn cpu_write(addr: u16, data: u8) {
        match addr {
            // Pulse 1
            0x4000 => {}
            0x4001 => {}
            0x4002 => {}
            0x4003 => {}

            // Pulse 2
            0x4004 => {}
            0x4005 => {}
            0x4006 => {}
            0x4007 => {}

            // Triangle
            0x4008 => {}
            0x4009 => {}
            0x400A => {}
            0x400B => {}

            // Noise
            0x400C => {}
            0x400D => {}
            0x400E => {}
            0x400F => {}

            // DMC
            0x4010 => {}
            0x4011 => {}
            0x4012 => {}
            0x4013 => {}

            // Unused
            0x4014 => {}

            // All - Channel enable and length counter
            0x4015 => {}

            // Unused
            0x4016 => {}

            // All - Frame counter
            0x4017 => {}
            _ => {}
        }
    }

    fn clock() {}

    fn reset() {}
}

fn pulse_wave_generator(duty_cycle: f32, harmonics: usize, frequency: f32, time: f32) -> f32 {
    let (mut a, mut b): (f32, f32) = (0.0, 0.0);
    let p: f32 = duty_cycle * std::f32::consts::TAU;
    for n in 1..=harmonics {
        let harmonic: f32 = n as f32;
        let c: f32 = harmonic * frequency * std::f32::consts::TAU * time;
        a += c.sin() / harmonic;
        b += (c - p * harmonic).sin() / harmonic;
    }
    SCALING * (a - b)
}

fn triangle_wave_generator() {}

fn noise_wave_generator() {}

fn dmc_wave_generator() {}
