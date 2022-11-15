const postCssPlugin = require("esbuild-style-plugin");
const esbuild = require("esbuild");

const args = process.argv.slice(2);
const watch = args.includes("--watch");
const deploy = args.includes("--deploy");

let opts = {
  entryPoints: ["css/app.css"],
  outdir: "../web/assets",
  bundle: true,
  sourcemap: "inline",
  plugins: [
    postCssPlugin({
      postcss: {
        plugins: [require("tailwindcss"), require("autoprefixer")],
      },
    }),
  ],
};

if (watch) {
  opts = {
    ...opts,
    watch,
  };
}

if (deploy) {
  opts = {
    ...opts,
    minify: true,
    sourcemap: false,
  };
}

const promise = esbuild.build(opts);

if (watch) {
  promise.then((_result) => {
    process.stdin.on("close", () => {
      process.exit(0);
    });

    process.stdin.resume();
  });
}
