import * as path from "jsr:@std/path@1.0.2";
import * as fs from "jsr:@std/fs@1.0.1";
import * as colors from "jsr:@std/fmt@1.0.0/colors";
import { printf } from "jsr:@std/fmt@1.0.0/printf";

const __dirname = path.dirname(path.fromFileUrl(import.meta.url));

async function main() {
  const mode = Deno.args[0];
  if (mode !== "release" && mode !== "debug") {
    console.error(colors.red("Mode must be 'release' or 'debug'"));
    Deno.exit(1);
  }
  const checker = new Checker(mode);

  const all_test_groups = [];
  if (Deno.args[1] !== undefined) {
    // When a test case is specified.
    const [group, name] = Deno.args[1].split("/") as [string, string?];
    if (name !== undefined) {
      const test_case = await get_test_case(group, name);
      all_test_groups.push({
        name: group,
        path: path.join(__dirname, "cases", group),
        cases: [test_case],
      });
    } else {
      const test_group = await get_test_group(group);
      all_test_groups.push(test_group);
    }
  } else {
    for await (
      const entry of fs.expandGlob(path.join(__dirname, "cases", "*"))
    ) {
      if (entry.isDirectory) {
        const test_group = await get_test_group(entry.name);
        all_test_groups.push(test_group);
      }
    }
  }
  all_test_groups.sort(() => 0.5 - Math.random()); // shuffle

  const count = { ok: 0, ng: 0 };
  const ng_results: Extract<TestResult, { status: "ng" }>[] = [];
  console.log(`\nRunning tests in ${colors.cyan(mode)} mode...\n`);
  for (const test_group of all_test_groups) {
    for await (const result of checker.test(test_group)) {
      if (result.status === "ok") {
        printf("[%s] %s\n", colors.green("ok"), result.name);
        count.ok += 1;
      } else {
        printf("[%s] %s\n", colors.red("ng"), result.name);
        count.ng += 1;
        ng_results.push(result);
      }
    }
  }

  printf("\n%s\n", colors.gray("Test result:"));
  printf("  %s: %d\n", colors.green("ok"), count.ok);
  printf("  %s: %d\n", colors.red("ng"), count.ng);
  if (count.ng == 0) {
    printf("\n%s\n", colors.green("All tests passed!"));
  } else {
    printf("\n%s\n", colors.red("Some tests failed."));
    const snip = function (s: string) {
      if (s.length > 100) {
        const rest = s.length - 100;
        return s.substring(0, 100) + colors.gray(` ... and ${rest} more`);
      } else {
        return s;
      }
    };
    for (const result of ng_results) {
      printf("\n%s\n", colors.inverse(result.name));
      printf(colors.gray(colors.italic("Expected:")));
      printf("\n%s\n", snip(result.expected));
      printf(colors.gray(colors.italic("Got:")));
      printf("\n%s\n", snip(result.actual));
      console.error(
        colors.gray("For more detail: ") +
          colors.bold(`deno task test ${mode} ${result.name}`),
      );
    }
    Deno.exit(1);
  }
}

interface TestCase {
  readonly name?: string;
  readonly input?: string;
  readonly output?: string;
}

type BuildMode = "release" | "debug";

interface TestGroup {
  readonly name: string;
  readonly path: string;
  readonly cases: TestCase[];
}

async function read_file_safe(path: string): Promise<string | undefined> {
  if (fs.existsSync(path)) {
    return await Deno.readTextFile(path);
  }
  return undefined;
}

async function get_test_case(group: string, name: string): Promise<TestCase> {
  const group_path = path.join(__dirname, "cases", group);
  const input_path = path.join(group_path, "input", `${name}.txt`);
  const output_path = path.join(group_path, "output", `${name}.txt`);
  return {
    name,
    input: await read_file_safe(input_path),
    output: await read_file_safe(output_path),
  };
}

async function get_test_group(group_name: string): Promise<TestGroup> {
  const group_path = path.join(__dirname, "cases", group_name);

  const input_txt = await read_file_safe(path.join(group_path, "input.txt"));
  const output_txt = await read_file_safe(path.join(group_path, "output.txt"));
  if (input_txt !== undefined || output_txt !== undefined) {
    return {
      name: group_name,
      path: group_path,
      cases: [{
        name: undefined,
        input: input_txt,
        output: output_txt,
      }],
    };
  }

  const all_test_names = await (async function () {
    const collect_test_names = async function (type: "input" | "output") {
      const iter = fs.expandGlob(path.join(group_path, type, "*.txt"));
      const entries = await Array.fromAsync(iter);
      return entries.map((x) => x.name.replace(/\.txt$/, ""));
    };
    const input_names = await collect_test_names("input");
    const output_names = await collect_test_names("output");
    const names = new Set([...input_names, ...output_names]);
    return Array.from(names);
  })();

  const test_cases: TestCase[] = [];
  for (const name of all_test_names) {
    const test_case = await get_test_case(group_name, name);
    test_cases.push(test_case);
  }
  test_cases.sort(() => 0.5 - Math.random()); // shuffle

  return {
    name: group_name,
    path: group_path,
    cases: test_cases,
  };
}

type TestResult = {
  readonly name: string;
  readonly status: "ok";
} | {
  readonly name: string;
  readonly status: "ng";
  readonly actual: string;
  readonly expected: string;
};

class Checker {
  readonly mode: BuildMode;
  readonly #lico_path: string;
  readonly #textDecoder = new TextDecoder();
  readonly #textEncoder = new TextEncoder();

  constructor(mode: BuildMode) {
    this.mode = mode;
    this.#lico_path = cargo_build(mode);
  }

  async *test(group: TestGroup): AsyncGenerator<TestResult> {
    const target = path.join(group.path, "main.lico");

    const make_name = function (name?: string) {
      if (name === undefined) {
        return group.name;
      } else {
        return `${group.name}/${name}`;
      }
    };

    for (const test_case of group.cases) {
      const output = await this.#run(target, test_case.input);
      if (output === (test_case.output ?? "")) {
        yield {
          name: make_name(test_case.name),
          status: "ok",
        };
      } else {
        yield {
          name: make_name(test_case.name),
          status: "ng",
          actual: output,
          expected: test_case.output ?? "",
        };
      }
    }
  }

  async #run(source_code: string, input?: string): Promise<string> {
    const prosess = new Deno.Command(
      this.#lico_path,
      {
        args: [
          "run",
          source_code,
        ],
        stdin: "piped",
        stdout: "piped",
        stderr: "piped",
      },
    ).spawn();

    if (input) {
      const writer = prosess.stdin.getWriter();
      await writer.write(this.#textEncoder.encode(input));
      await writer.close();
    }

    const { code, stdout, stderr } = await prosess.output();
    if (code != 0) {
      console.error(
        colors.red(
          `\nWhile running ${source_code}, exit code is expected 0 but got ${code}`,
        ),
      );
    }
    if (stderr.length != 0) {
      const err = this.#textDecoder.decode(stderr);
      console.error(colors.italic(colors.gray("stderr:")));
      console.error(err);
    }
    return this.#textDecoder.decode(stdout);
  }
}

/**
 * Build `lico` and return the path to the executable.
 */
function cargo_build(mode: BuildMode): string {
  const cli_path = path.join(__dirname, "..", "cli");
  const cargo_build = new Deno.Command("cargo", {
    args: [
      "build",
      mode == "release" ? "--release" : undefined,
      "--manifest-path",
      path.join(cli_path, "Cargo.toml"),
    ].filter((x) => x !== undefined) as string[],
    stdout: "inherit",
    stderr: "inherit",
  });
  const { code } = cargo_build.outputSync();
  if (code != 0) {
    console.error(
      `Exit code of \`cargo build\` is expected 0 but got ${code}`,
    );
    Deno.exit(1);
  }
  return path.join(
    cli_path,
    "target",
    mode == "release" ? "release" : "debug",
    "lico",
  );
}

await main();
