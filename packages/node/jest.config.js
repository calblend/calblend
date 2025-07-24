/** @type {import('jest').Config} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: ['**/__tests__/**/*.test.ts'],
  collectCoverageFrom: [
    'src/**/*.ts',
    '!src/**/*.d.ts',
    '!src/__tests__/**',
  ],
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html'],
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
  transform: {
    '^.+\\.ts$': ['ts-jest', {
      tsconfig: {
        noUnusedLocals: false,
        noUnusedParameters: false,
      },
    }],
  },
  // Allow running tests with --expose-gc for memory leak detection
  globals: {
    'ts-jest': {
      isolatedModules: true,
    },
  },
  testTimeout: 30000, // 30 seconds for FFI operations
};