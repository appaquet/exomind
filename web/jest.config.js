module.exports = {
    preset: 'ts-jest',
    roots: ['<rootDir>/src'],
    transform: {
      '^.+\\.(tsx?|jsx?)$': 'ts-jest',
    },
    testRegex: '(/__tests__/.*|(\\.|/)(test|spec))\\.tsx?$',
    moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node'],
    automock: false,
  }
  