import { writeFileSync } from 'node:fs';
import { theme, version } from 'antdv-next';

const { defaultAlgorithm, darkAlgorithm, compactAlgorithm } = theme;

const dumps = {
  'tokens-light.json': theme.getDesignToken({ algorithm: defaultAlgorithm }),
  'tokens-dark.json': theme.getDesignToken({ algorithm: darkAlgorithm }),
  'tokens-compact.json': theme.getDesignToken({ algorithm: compactAlgorithm }),
  // 自定义主色：用来校验 Rust 侧复刻的色板派生算法是否一致
  'tokens-accent-violet.json': theme.getDesignToken({ token: { colorPrimary: '#8b5cf6' } }),
  'tokens-accent-violet-dark.json': theme.getDesignToken({
    algorithm: darkAlgorithm,
    token: { colorPrimary: '#8b5cf6' },
  }),
  'tokens-accent-green.json': theme.getDesignToken({ token: { colorPrimary: '#22c55e' } }),
};

console.log('antdv-next version:', version ?? '(unknown)');
for (const [name, token] of Object.entries(dumps)) {
  const flat = Object.fromEntries(
    Object.entries(token).map(([k, v]) => [k, v instanceof Function ? '(fn)' : v]),
  );
  writeFileSync(`./${name}`, JSON.stringify(flat, null, 1));
  console.log(`${name}: ${Object.keys(flat).length} tokens`);
}
