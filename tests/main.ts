import * as path from "https://deno.land/std@0.208.0/path/mod.ts";
import * as fs from "https://deno.land/std@0.208.0/fs/mod.ts";
import * as colors from "https://deno.land/std@0.208.0/fmt/colors.ts";
import { printf } from "https://deno.land/std@0.208.0/fmt/printf.ts";

const __dirname = path.dirname(path.fromFileUrl(import.meta.url));

async function main() {
  const mode = Deno.args[0];
  if (mode !== "release" && mode !== "debug") {
    console.error(colors.red("Mode must be 'release' or 'debug'"));
    Deno.exit(1);
  }

  const run = await build_run_command(mode);

  // Run a specific test case
  if (Deno.args[1] !== undefined) {
    const [group, name] = Deno.args[1].split("/");
    const test_case = (await get_test_cases(group)).find((x) => x.name == name);
    if (test_case === undefined) {
      printf(
        colors.red("Test case %s in %s not found.\n"),
        colors.bold(name),
        colors.bold(group),
      );
      Deno.exit(1);
    }
    const output = await run(test_case.path, test_case.input);
    if (output != test_case.output ?? "") {
      console.error(colors.red("Failed"));
      console.error(colors.gray(colors.italic("Output:")));
      console.error(output);
      Deno.exit(1);
    } else {
      console.log(colors.green("Passed"));
    }
    return;
  }

  console.log(`\nRunning tests in ${colors.cyan(mode)} mode...\n`);

  const test_all = async function (group: string) {
    const ngs: { name: string; actual: string; expected: string }[] = [];

    printf("%s\n", colors.gray(`Running test cases of '${group}'`));
    for (const test_case of await get_test_cases(group)) {
      printf("%s ...", test_case.name);
      const output = await run(test_case.path, test_case.input);
      if (output != test_case.output ?? "") {
        printf(colors.red(" ng\n"));
        ngs.push({
          name: test_case.name,
          actual: output,
          expected: test_case.output ?? "",
        });
      } else {
        printf(colors.green(" ok\n"));
      }
    }

    if (ngs.length > 0) {
      console.error(colors.red("\nFailed test cases:"));
      const snip = function (s: string) {
        if (s.length > 100) {
          const rest = s.length - 100;
          return s.substring(0, 100) + colors.gray(` ... and ${rest} more`);
        } else {
          return s;
        }
      };
      for (const ng of ngs) {
        console.error();
        console.error(colors.inverse(ng.name));
        console.error(colors.gray(colors.italic("Expected:")));
        console.error(snip(ng.expected));
        console.error(colors.gray(colors.italic("Got:")));
        console.error(snip(ng.actual));
        console.error(
          colors.gray("For more detail: ") +
            colors.bold(`deno task test ${mode} ${group}/${ng.name}`),
        );
      }
      Deno.exit(1);
    }
  };

  await test_all("random");
}

async function get_test_cases(group: string) {
  const test_cases: {
    name: string;
    path: string;
    input?: string;
    output?: string;
  }[] = [];

  for await (
    const file of fs.expandGlob(path.join(__dirname, group, "*.lico"))
  ) {
    const get = function (path: string) {
      if (fs.existsSync(path)) {
        return Deno.readTextFileSync(path);
      }
      return undefined;
    };

    const name = file.name.replace(/\.lico$/, "");
    test_cases.push({
      name,
      path: file.path,
      input: get(path.join(__dirname, group, "in", name + ".txt")),
      output: get(path.join(__dirname, group, "out", name + ".txt")),
    });
  }
  test_cases.sort((_, __) => 0.5 - Math.random()); // shuffle
  return test_cases;
}

async function build_run_command(mode: "release" | "debug") {
  const cli_path = path.join(__dirname, "..", "cli");

  const build_command = new Deno.Command("cargo", {
    args: [
      "build",
      mode == "release" ? "--release" : undefined,
      "--manifest-path",
      path.join(cli_path, "Cargo.toml"),
    ].filter((x) => x !== undefined) as string[],
    stdout: "inherit",
    stderr: "inherit",
  });
  const { code } = await build_command.output();
  if (code != 0) {
    console.error(`Exit code of \`cargo build\` is expected 0 but got ${code}`);
  }

  const textDecoder = new TextDecoder();
  const textEncoder = new TextEncoder();
  return async function (src: string, input?: string): Promise<string> {
    const prosess = new Deno.Command(
      path.join(
        cli_path,
        "target",
        mode == "release" ? "release" : "debug",
        "lico",
      ),
      {
        args: [
          "run",
          src,
        ],
        stdin: "piped",
        stdout: "piped",
        stderr: "piped",
      },
    ).spawn();

    if (input) {
      const writer = prosess.stdin.getWriter();
      await writer.write(textEncoder.encode(input));
      await writer.close();
    }

    const { code, stdout, stderr } = await prosess.output();
    if (code != 0) {
      console.error(
        colors.red(
          `\nWhile running ${src}, exit code is expected 0 but got ${code}`,
        ),
      );
    }
    if (stderr.length != 0) {
      const err = textDecoder.decode(stderr);
      console.error(colors.italic(colors.gray("stderr:")));
      console.error(err);
    }
    return textDecoder.decode(stdout);
  };
}

await main();
