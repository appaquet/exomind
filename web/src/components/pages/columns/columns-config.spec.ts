
import { ColumnsConfig, ColumnConfig } from './columns-config';

describe('ColumnsConfig', () => {
  it('should be able to load and export to string', () => {
    const cf = ColumnsConfig.fromString('l-2-3');
    expect(cf.parts.length).toBe(3);
    expect(cf.toString()).toBe('l-2-3');
  });

  it('should handle empty state', () => {
    const cf = ColumnsConfig.fromString('');
    expect(cf.empty).toBeTruthy();
    expect(cf.toString()).toBe('');
  });

  it('should handle set', () => {
    const cf = ColumnsConfig.fromString('1-2-3-4-5-6-7-8');
    expect(cf.set(5, new ColumnConfig('10')).toString()).toBe('1-2-3-4-5-10');
  });

  it('should handle unset', () => {
    const cf = ColumnsConfig.fromString('1-2-3-4-5-6-7-8');
    expect(cf.unset(5).toString()).toBe('1-2-3-4-5');
  });

  it('should handle all kind of column type', () => {
    const cf = ColumnsConfig.fromString('i-c-s-o2');
    expect(cf.parts[0].isInbox).toBeTruthy();
    expect(cf.parts[1].isCollection).toBeTruthy();
    expect(cf.parts[2].isSearch).toBeTruthy();
    expect(cf.parts[3].isObject).toBeTruthy();
  });

  it('should have an url encoded toString', () => {
    const cf = new ColumnsConfig([new ColumnConfig('s23122')]);
    expect(cf.toString()).not.toContain('-');
  });

  it('should be able to extract value from column', () => {
    const cf = new ColumnConfig('shello');
    expect(cf.token).toBe('s');
    expect(cf.value).toBe('hello');
  });

  it('should be able to store extra information', () => {
    const cf = new ColumnConfig('shello');
    expect(cf.value).toBe('hello');
    expect(cf.extra).toBeNull();

    const newCf = cf.withExtra('bob');
    expect(cf.value).toBe('hello');
    expect(cf.extra).toBeNull();
    expect(newCf.value).toBe('hello');
    expect(newCf.extra).toBe('bob');
  });
});
