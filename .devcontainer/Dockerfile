FROM rust:latest

# Configure apt and install packages
RUN apt-get update \
    && apt-get -y install --no-install-recommends apt-utils dialog 2>&1 \
    #
    # Verify git, process tools installed
    && apt-get -y install git iproute2 procps lsb-release \
    && apt-get install -y libgfortran5 libopenblas-dev gcc g++ gfortran \
    #
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Expose port for web server
EXPOSE 8000

ENV PATH="/root/.cargo/bin:${PATH}"
