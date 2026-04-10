# Rename the built dist file
mv ./dist/kobold_qrcode_example.js ./dist/kobold_qrcode_example_large.js
# Minimize the dist file into a new one
esbuild --bundle ./dist/kobold_qrcode_example_large.js --outfile=./dist/kobold_qrcode_example.js --format=esm
