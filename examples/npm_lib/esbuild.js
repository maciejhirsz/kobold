// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

import esbuild from 'esbuild';

// convert to ECMAScript module (ESM) format with extension .mjs
// for usage with `import x from y;` and use `"type": "module"`
// in package.json to allow Node.js to interpret the files as ESM files
esbuild.build({
    entryPoints: ['koboldNpmLib.js'],
    bundle: true,
    outfile: 'output/koboldNpmLib.mjs',
    format: 'esm',
    minify: false,
}).catch(() => process.exit(1));
