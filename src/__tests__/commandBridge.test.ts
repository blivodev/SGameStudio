import { describe, it, expect } from 'vitest';
import { buildDispatchBody } from '../services/commandBridge';

describe('commandBridge', () => {
  it('builds a dispatch body with command and payload', () => {
    const body = buildDispatchBody('create_3d_primitive', { primitiveType: 'cube' });
    const parsed = JSON.parse(body);
    expect(parsed.command).toBe('create_3d_primitive');
    expect(parsed.payload.primitiveType).toBe('cube');
  });
});