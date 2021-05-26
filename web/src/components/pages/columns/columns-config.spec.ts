
import { ColumnConfigs, ColumnConfig, SearchToken } from './columns-config';

describe('ColumnsConfig', () => {
  it('should be able to load and export to string', () => {
    const cf = ColumnConfigs.fromString('l-2-3');
    expect(cf.parts.length).toBe(3);
    expect(cf.toString()).toBe('l-2-3');
  });

  it('should handle empty state', () => {
    const cf = ColumnConfigs.fromString('');
    expect(cf.empty).toBeTruthy();
    expect(cf.toString()).toBe('');
  });

  it('should handle set', () => {
    const cf = ColumnConfigs.fromString('s1-s2-s3-s4-s5-s6-s7-s8');
    expect(cf.set(5, ColumnConfig.fromString('s10')).toString()).toBe('s1-s2-s3-s4-s5-s10');
  });

  it('should handle unset', () => {
    const cf = ColumnConfigs.fromString('1-2-3-4-5-6-7-8');
    expect(cf.unset(5).toString()).toBe('1-2-3-4-5');
  });

  it('should handle all kind of column type', () => {
    const cf = ColumnConfigs.fromString('i-c-s-o2');
    expect(cf.parts[0].isInbox).toBeTruthy();
    expect(cf.parts[1].isCollection).toBeTruthy();
    expect(cf.parts[2].isSearch).toBeTruthy();
    expect(cf.parts[3].isObject).toBeTruthy();
  });

  it('should have an url encoded toString', () => {
    const cf = new ColumnConfigs([new ColumnConfig(SearchToken, '23122')]);
    expect(cf.toString()).not.toContain('-');
  });

  it('should be able extract column from string', () => {
    const cf = ColumnConfig.fromString('shello');
    expect(cf.type).toBe('s');
    expect(cf.first).toBe('hello');
  });

  it('should be able to store extra information', () => {
    const cf = new ColumnConfig(SearchToken, 'hello');
    expect(cf.first).toBe('hello');
    expect(cf.second).toBeNull();

    const newCf = cf.withExtra('bob');
    expect(cf.first).toBe('hello');
    expect(cf.second).toBeNull();
    expect(newCf.first).toBe('hello');
    expect(newCf.second).toBe('bob');
  });
});
