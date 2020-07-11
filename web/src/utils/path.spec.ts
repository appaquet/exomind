
import Path from './path';

describe('path', () => {
  it('adds 1 + 2 to equal 3', () => {
    const p = new Path('test1/test2');
    expect(p.toString()).toBe('test1/test2');
  });
});
