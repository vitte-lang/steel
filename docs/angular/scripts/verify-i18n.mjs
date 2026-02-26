#!/usr/bin/env node
import { readFileSync } from 'node:fs';

const file = 'src/app/app.state.ts';
const text = readFileSync(file, 'utf8');
const i18nBlock = text.match(/const I18N: Record<LangKey, I18n> = \{[\s\S]*?\n};\n\nconst HUMANIZED_COPY_SUFFIX/);
if (!i18nBlock) {
  throw new Error('Could not locate I18N runtime block');
}
const i18nRuntime = i18nBlock[0];

const LOCALE_COUNT = 9;
const requiredDownloadKeys = ['vscodeTitle:', 'vscodeDesc:', 'vscodeButton:'];
for (const key of requiredDownloadKeys) {
  const matches = i18nRuntime.match(new RegExp(key, 'g')) ?? [];
  if (matches.length !== LOCALE_COUNT) {
    throw new Error(`Expected ${LOCALE_COUNT} occurrences of '${key}', found ${matches.length}`);
  }
}

const localeKeys = ['en', 'fr', 'de', 'it', 'ar', 'zh', 'ja', 'pt', 'es'];
const suffixBlock = text.match(/const HUMANIZED_COPY_SUFFIX:[\s\S]*?};\n\nfor \(const lang of Object\.keys\(I18N\)/);
if (!suffixBlock) {
  throw new Error('Missing HUMANIZED_COPY_SUFFIX block');
}
for (const key of localeKeys) {
  if (!new RegExp(`\\b${key}:\\s*\\{`).test(suffixBlock[0])) {
    throw new Error(`Missing HUMANIZED_COPY_SUFFIX locale '${key}'`);
  }
}

const appendix = text.match(/const COPY_READY_APPENDIX = `([\s\S]*?)`;/);
if (!appendix) {
  throw new Error('Missing COPY_READY_APPENDIX');
}
const appendixBody = appendix[1];
const snapshot = [
  '\\n\\n;; Copy-ready checklist',
  ';; 1) Keep workspace/root/target_dir as-is for first run.',
  ';; 2) Rename tool executables only if your machine uses different names.',
  ';; 3) Start with one bake, then split into debug/release as needed.',
  ';; 4) Keep outputs under target/out so cleanup stays easy.',
  ';; 5) Run: steel --version && steel run'
].join('\\n');
if (!appendixBody.includes(snapshot)) {
  throw new Error('COPY_READY_APPENDIX snapshot mismatch');
}

if (!text.includes('...withCopyReadyExamples(CORE_EXAMPLES)') || !text.includes('...withCopyReadyExamples(EXTRA_EXAMPLES)')) {
  throw new Error('Examples are not wrapped with withCopyReadyExamples');
}

if (text.includes('\\t')) {
  throw new Error('Found literal \\t in app.state.ts; examples should use 2-space indentation');
}

console.log('i18n/snapshot checks passed');
