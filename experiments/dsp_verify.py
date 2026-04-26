import numpy as np
import scipy.signal as signal
import matplotlib.pyplot as plt
import sys
import argparse

def calc_peaking_eq(freq, q, gain_db, fs=48000):
    """
    Calculates biquad coefficients for a peaking equalizer.
    Matches common Audio EQ Cookbook formulas.
    """
    A = 10**(gain_db / 40)
    omega = 2 * np.pi * freq / fs
    sn = np.sin(omega)
    cs = np.cos(omega)
    alpha = sn / (2 * q)

    b0 = 1 + alpha * A
    b1 = -2 * cs
    b2 = 1 - alpha * A
    a0 = 1 + alpha / A
    a1 = -2 * cs
    a2 = 1 - alpha / A

    # Normalize by a0
    return [b0/a0, b1/a0, b2/a0, a1/a0, a2/a0]

def plot_response(coeffs, fs=48000):
    b = coeffs[:3]
    a = [1.0] + coeffs[3:]
    w, h = signal.freqz(b, a, worN=8000)
    
    plt.figure(figsize=(10, 6))
    plt.semilogx(w * fs / (2 * np.pi), 20 * np.log10(abs(h)))
    plt.title("Filter Frequency Response")
    plt.xlabel("Frequency [Hz]")
    plt.ylabel("Amplitude [dB]")
    plt.grid(which='both', axis='both')
    plt.axhline(0, color='black', lw=1)
    plt.ylim(-20, 20)
    plt.savefig("experiments/dsp_response.png")
    print("Plot saved to experiments/dsp_response.png")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--freq", type=float, required=True)
    parser.add_argument("--q", type=float, required=True)
    parser.add_argument("--gain", type=float, required=True)
    parser.add_argument("--fs", type=float, default=48000)
    args = parser.parse_args()

    coeffs = calc_peaking_eq(args.freq, args.q, args.gain, args.fs)
    print(f"Coefficients for Freq={args.freq}, Q={args.q}, Gain={args.gain}:")
    print(f"b0: {coeffs[0]:.6f}, b1: {coeffs[1]:.6f}, b2: {coeffs[2]:.6f}")
    print(f"a1: {coeffs[3]:.6f}, a2: {coeffs[4]:.6f}")
    
    plot_response(coeffs, args.fs)
