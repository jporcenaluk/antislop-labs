import { describe, it, expect } from 'vitest';
import { formatTime } from './timer';

describe('formatTime', () => {
	it('formats zero seconds', () => {
		expect(formatTime(0)).toBe('00:00');
	});

	it('formats seconds only', () => {
		expect(formatTime(45)).toBe('00:45');
	});

	it('formats minutes and seconds', () => {
		expect(formatTime(125)).toBe('02:05');
	});

	it('formats 25 minutes', () => {
		expect(formatTime(25 * 60)).toBe('25:00');
	});

	it('formats large values', () => {
		expect(formatTime(90 * 60 + 30)).toBe('90:30');
	});

	it('pads single digits', () => {
		expect(formatTime(61)).toBe('01:01');
	});
});
