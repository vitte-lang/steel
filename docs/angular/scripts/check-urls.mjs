#!/usr/bin/env node
import { readFileSync } from 'node:fs';

const file = 'src/app/app.state.ts';
const text = readFileSync(file, 'utf8');

function parseConstantUrl(name) {
  const match = text.match(new RegExp(`const ${name} =\\s*'([^']+)'`));
  if (!match) {
    throw new Error(`Missing ${name} constant value`);
  }
  let url;
  try {
    url = new URL(match[1]);
  } catch {
    throw new Error(`${name} is not a valid URL`);
  }
  if (url.protocol !== 'https:') {
    throw new Error(`${name} must use https`);
  }
  return url;
}

if (!text.includes("const DOWNLOAD_RELEASES_URL =")) {
  throw new Error('Missing DOWNLOAD_RELEASES_URL constant');
}
if (!text.includes("const VSCODE_EXTENSION_URL =")) {
  throw new Error('Missing VSCODE_EXTENSION_URL constant');
}
if (!text.includes("readonly downloadUrl = DOWNLOAD_RELEASES_URL;")) {
  throw new Error('downloadUrl must reference DOWNLOAD_RELEASES_URL');
}
if (!text.includes("readonly vscodeExtensionUrl = VSCODE_EXTENSION_URL;")) {
  throw new Error('vscodeExtensionUrl must reference VSCODE_EXTENSION_URL');
}

const releasesUrl = parseConstantUrl('DOWNLOAD_RELEASES_URL');
if (releasesUrl.pathname === '/vitte-lang/steel.org/releases/') {
  throw new Error('DOWNLOAD_RELEASES_URL must not use a trailing slash (/releases/)');
}
if (releasesUrl.hostname !== 'github.com' || releasesUrl.pathname !== '/vitte-lang/steel.org/releases') {
  throw new Error('DOWNLOAD_RELEASES_URL must point exactly to https://github.com/vitte-lang/steel.org/releases');
}

const vscodeUrl = parseConstantUrl('VSCODE_EXTENSION_URL');
if (
  vscodeUrl.hostname !== 'marketplace.visualstudio.com' ||
  vscodeUrl.pathname !== '/items' ||
  vscodeUrl.searchParams.get('itemName') !== 'steelcommand.steel-command'
) {
  throw new Error(
    'VSCODE_EXTENSION_URL must point exactly to https://marketplace.visualstudio.com/items?itemName=steelcommand.steel-command'
  );
}

console.log('download/vscode URLs checks passed');
