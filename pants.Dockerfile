FROM python:3.7-slim-buster as pants-base

SHELL ["/bin/bash", "-c"]

RUN apt-get update && apt-get install --yes \
    build-essential \
    curl \
    unzip

RUN adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl

USER grapl
ENV USER=grapl
ENV PANTS_CONFIG_FILES="['pants.toml', 'pants.ci.toml']"
WORKDIR /home/grapl/workdir

# These file are currently needed in order to execute any pants command.
COPY --chown=grapl pants pants
COPY --chown=grapl pants-plugins pants-plugins
COPY --chown=grapl pyproject.toml pyproject.toml
COPY --chown=grapl src/python/mypy.ini src/python/mypy.ini
COPY --chown=grapl .flake8 .flake8
COPY --chown=grapl pants.toml pants.toml
COPY --chown=grapl pants.ci.toml pants.ci.toml
COPY --chown=grapl 3rdparty 3rdparty

RUN ./pants --version

FROM pants-base AS base

COPY --chown=grapl src/proto src/proto
COPY --chown=grapl src/python src/python
