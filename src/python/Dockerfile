# base build
################################################################################

FROM python:3.7-slim-buster AS grapl-python-build

SHELL ["/bin/bash", "-c"]

RUN apt-get update && apt-get -y install --no-install-recommends \
        build-essential \
        protobuf-compiler \
        zip \
    && rm -rf /var/lib/apt/lists/*

ENV PROTOC /usr/bin/protoc
ENV PROTOC_INCLUDE /usr/include

RUN adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl

USER grapl
ENV USER=grapl
WORKDIR /home/grapl

# Automatically ensures that our virtualenv is created and active on
# all subsequent actions
ENV VIRTUAL_ENV=/home/grapl/venv
RUN python3 -m venv $VIRTUAL_ENV
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

RUN pip install --upgrade pip && \
    pip install wheel grpcio chalice hypothesis pytest pytest-xdist

# base deploy
################################################################################

FROM python:3.7-slim-buster AS grapl-python-deploy

SHELL ["/bin/bash", "-c"]

RUN apt-get update && apt-get -y install --no-install-recommends \
        bash \
        libstdc++6 \
    && rm -rf /var/lib/apt/lists/*

RUN adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl

USER grapl
WORKDIR /home/grapl

# Automatically ensures that our virtualenv is created and active on
# all subsequent actions!
ENV VIRTUAL_ENV=/home/grapl/venv
RUN python3 -m venv $VIRTUAL_ENV
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

RUN pip install --upgrade pip

# test deps
################################################################################

FROM grapl-python-build AS python-test-deps

COPY --chown=grapl src/python/python_test_deps python_test_deps

RUN python_test_deps/download_requirements.sh



# graplinc-grapl-api
################################################################################

# build
FROM grapl-python-build AS graplinc-grapl-api-build

COPY --chown=grapl src/proto proto
COPY --chown=grapl src/python/graplinc-grapl-api python/graplinc-grapl-api

RUN cd python/graplinc-grapl-api && \
    python setup.py sdist bdist_wheel && \
    pip install dist/*.whl

# grapl_analyzerlib
################################################################################

# build
FROM grapl-python-build AS grapl-analyzerlib-build

COPY --chown=grapl src/python/grapl_analyzerlib grapl_analyzerlib
COPY --chown=grapl --from=graplinc-grapl-api-build /home/grapl/venv venv

RUN --mount=type=bind,source=dist,target=/grapl-dist cd /grapl-dist && \
    pip install grapl_common-*.whl

RUN cd grapl_analyzerlib && \
    pip install . && \
    python setup.py sdist bdist_wheel

# test
FROM grapl-analyzerlib-build AS grapl-analyzerlib-test

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh


# analyzer-executor
################################################################################

# build
FROM grapl-python-build AS analyzer-executor-build

COPY --chown=grapl src/python/analyzer_executor analyzer_executor
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN cd analyzer_executor && \
    pip install .

# deploy
FROM grapl-python-deploy AS analyzer-executor-deploy

COPY --chown=grapl --from=analyzer-executor-build /home/grapl/venv venv
COPY --chown=grapl --from=analyzer-executor-build /home/grapl/analyzer_executor analyzer_executor

CMD python3 analyzer_executor/src/run.py

# test
FROM analyzer-executor-build AS analyzer-executor-test

ENV IS_RETRY=False

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

# model-plugin-deployer
################################################################################

# build
FROM grapl-python-build AS model-plugin-deployer-build

COPY --chown=grapl src/python/grapl-model-plugin-deployer model-plugin-deployer
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN cd model-plugin-deployer && \
    pip install -r requirements.txt

# deploy
FROM grapl-python-deploy AS model-plugin-deployer-deploy

COPY --chown=grapl --from=model-plugin-deployer-build /home/grapl/venv venv

RUN chalice new-project app/

COPY --chown=grapl --from=model-plugin-deployer-build /home/grapl/model-plugin-deployer/grapl_model_plugin_deployer.py app/app.py

CMD cd app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8123

# Notebook
################################################################################

# build
FROM grapl-python-build AS grapl-notebook

SHELL ["/bin/bash", "-o", "pipefail", "-c"]
EXPOSE 8888

COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN pip install jupyter

# Set up jupyter-notebook stuff
RUN mkdir -p grapl-notebook/model_plugins
COPY --chown=grapl src/python/grapl-notebook/jupyter_notebook_config.py /home/grapl/.jupyter/
COPY --chown=grapl src/python/grapl-notebook/Demo_Engagement.ipynb grapl-notebook/

## Run it
CMD cd grapl-notebook && \
    jupyter notebook --ip="0.0.0.0"

# test
FROM grapl-notebook AS grapl-notebook-test

# This file will improve once #444 lands
# primarily to use pre-downloaded tools instead of downloading again

RUN pip install nbqa mypy boto3-stubs[essential]

# grapl-tests-common-build
################################################################################
FROM grapl-python-build AS grapl-tests-common-build

COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv
COPY --from=python-test-deps /home/grapl/python_test_deps python_test_deps

RUN python_test_deps/install_requirements.sh

COPY --chown=grapl src/python/grapl-tests-common grapl-tests-common
RUN cd grapl-tests-common && \
    pip install . && \
    python setup.py sdist bdist_wheel

# GraphQL Endpoint tests
################################################################################
FROM grapl-tests-common-build AS graphql-endpoint-tests

COPY --chown=grapl src/python/graphql_endpoint_tests graphql_endpoint_tests
