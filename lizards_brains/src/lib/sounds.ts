let audioContext: AudioContext | null = null;

function getAudioContext(): AudioContext {
	if (!audioContext) {
		audioContext = new AudioContext();
	}
	return audioContext;
}

export async function playCompletionSound(): Promise<void> {
	try {
		const ctx = getAudioContext();

		// Generate a pleasant chime using Web Audio API
		const now = ctx.currentTime;

		// First tone
		const osc1 = ctx.createOscillator();
		const gain1 = ctx.createGain();
		osc1.type = 'sine';
		osc1.frequency.setValueAtTime(880, now);
		gain1.gain.setValueAtTime(0.3, now);
		gain1.gain.exponentialRampToValueAtTime(0.01, now + 0.5);
		osc1.connect(gain1);
		gain1.connect(ctx.destination);
		osc1.start(now);
		osc1.stop(now + 0.5);

		// Second tone (higher, slight delay)
		const osc2 = ctx.createOscillator();
		const gain2 = ctx.createGain();
		osc2.type = 'sine';
		osc2.frequency.setValueAtTime(1320, now + 0.15);
		gain2.gain.setValueAtTime(0, now);
		gain2.gain.setValueAtTime(0.25, now + 0.15);
		gain2.gain.exponentialRampToValueAtTime(0.01, now + 0.7);
		osc2.connect(gain2);
		gain2.connect(ctx.destination);
		osc2.start(now + 0.15);
		osc2.stop(now + 0.7);
	} catch (e) {
		console.warn('Failed to play completion sound:', e);
	}
}
