#!/usr/bin/env python3

"""Encapsulates logic for generating and updating editor
configuration files to make it easy to work with Grapl code.

Provided as a self-documenting Click app for discoverability and ease
of maintenance.

"""

import json
from typing import Dict, List, Union

import click
import toml
from typing_extensions import TypedDict


# NOTE: This is essentially to silence the typechecker (and help us
# not shoot ourselves in the foot). It's not making any attempt to be
# a complete and faithful typing of Pyright configuration documents;
# it's just typing what we're currently using. Feel free to update
# this as this code develops and matures.
class PyrightConfig(TypedDict):
    pythonVersion: str
    pythonPlatform: str
    venvPath: str
    venv: str
    verboseOutput: bool
    reportMissingImports: bool
    exclude: List[str]
    executionEnvironments: List[Dict[str, Union[str, List[str]]]]


BASE_PYRIGHTCONFIG: PyrightConfig = {
    "pythonVersion": "3.7",
    "pythonPlatform": "Linux",
    "venvPath": "build-support",
    "venv": "grapl-venv",
    "verboseOutput": True,
    "reportMissingImports": True,
    "exclude": [
        "src/js/**",
        "src/rust/**",
    ],
    "executionEnvironments": [
        {"root": "pulumi"},
        {"root": "pants-plugins"},
        # NOTE: We will augment this with the src/python root in the
        # code below
    ],
}

PANTS_TOML = "pants.toml"
PYRIGHTCONFIG_JSON = "pyrightconfig.json"


def src_python_execution_environment() -> Dict[str, Union[str, List[str]]]:
    """Generate a pyright "executionEnvironments" entry for code in our
    `src/python` directory.

    Since this code is all interrelated, we need to provide the
    appropriate "extraPaths" for Pyright to properly resolve imports,
    types, etc. In general, this amounts to adding our Pants source
    roots, with a few caveats:

    1) not all the roots are required for Python code in that
    directory

    2) Our Pants configuration explicitly provides absolute paths, not
    patterns that may be matched anywhere

    As such, we first filter out what we don't need, and then
    "relativize" the paths, since this is what Pyright need.

    """
    pants = toml.load(PANTS_TOML)
    source_roots = pants["source"]["root_patterns"]

    if any(not r.startswith("/") for r in source_roots):
        raise click.ClickException(
            "Expected all Pants source roots to be absolute, but at least one was not!"
        )

    # We don't care about these source roots for things that are in src/python
    filtered = [
        root
        for root in source_roots
        if root
        not in (
            "/3rdparty",
            "/build-support",
            "/pants-plugins",
            "/pulumi",
            "/src/aws-provision",
            "/src/proto",
        )
    ]
    relativized = [root.lstrip("/") for root in filtered]
    relativized.append("dist/codegen")

    return {"root": "src/python", "extraPaths": relativized}


def write_or_echo(output: str, path: str, write_file: bool) -> None:
    """ Consolidate logic for whether to write `output` to the file at `path`, or to send it to standard output instead."""
    if write_file:
        with click.open_file(path, "w") as f:
            f.write(output)
        click.echo(f"Wrote content to {path} file")
    else:
        click.echo(output)


@click.command(name="generate")
@click.option(
    "--write-file/--no-write-file",
    is_flag=True,
    default=True,
    show_default=True,
    help="Controls whether or not to write the generated output to disk, or to standard output.",
)
def generate_pyrightconfig(write_file: bool) -> None:
    """Generate a pyrightconfig.json file from pants.toml.

    Do this if you have no existing pyrightconfig.json file that you
    are using. If you already have one, on the other hand, please see
    the `update` command instead.

    """

    pyrightconfig = BASE_PYRIGHTCONFIG
    pyrightconfig["executionEnvironments"].append(src_python_execution_environment())
    output = json.dumps(pyrightconfig, indent=4)

    write_or_echo(output, PYRIGHTCONFIG_JSON, write_file)


@click.command(name="update")
@click.option(
    "--write-file/--no-write-file",
    is_flag=True,
    default=True,
    show_default=True,
    help="Controls whether or not to write the generated output to disk, or to standard output.",
)
def update_pyrightconfig(write_file: bool) -> None:
    """Update an existing pyrightconfig.json file.

    In particular, the `extraPaths` entries for various
    `executionEnvironments` must be kept in-sync with what we declare
    in our pants.toml file.

    Any other changes you may have made to your file will be
    preserved.

    """

    with click.open_file(PYRIGHTCONFIG_JSON, "r") as f:
        pyright = json.load(f)

    execution_environments = pyright["executionEnvironments"]

    # Preserve other environments; we're only concerned about the
    # src/python one here
    new_execution_environments = [
        e for e in execution_environments if e["root"] != "src/python"
    ]
    new_execution_environments.append(src_python_execution_environment())
    pyright.update({"executionEnvironments": new_execution_environments})

    output = json.dumps(pyright, indent=4)
    write_or_echo(output, PYRIGHTCONFIG_JSON, write_file)


@click.group(name="pyright")
def configure_pyright() -> None:
    """ Set up Pyright for Python IDE integration. """


configure_pyright.add_command(generate_pyrightconfig)
configure_pyright.add_command(update_pyrightconfig)


@click.group()
def editor_setup() -> None:
    """A utility for helping to configure IDEs and editors for working
    with Grapl code."""


editor_setup.add_command(configure_pyright)

if __name__ == "__main__":
    editor_setup()
