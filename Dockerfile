# Use an official Rust runtime as a parent image
FROM rust:latest

# Make port 8000 available to the world outside this container
EXPOSE 8000

RUN apt-get update && \
    apt-get install -y libgfortran5 libopenblas-dev gcc g++ gfortran

RUN find / -name libgfortran*.so* 
RUN find / -name libopenblas*.so*

WORKDIR /usr/src/myapp

COPY . .

# Run a script to find the libraries and set RUSTFLAGS
RUN GFORTRAN_PATH=$(dirname $(find / -name libgfortran*.so* | head -n 1)) && \
    OPENBLAS_PATH=$(dirname $(find / -name libopenblas*.so* | head -n 1)) && \
    echo "GFORTRAN_PATH=$GFORTRAN_PATH" >> /usr/src/myapp/.env && \
    echo "OPENBLAS_PATH=$OPENBLAS_PATH" >> /usr/src/myapp/.env && \
    echo 'export RUSTFLAGS="-C link-arg=-L$GFORTRAN_PATH -C link-arg=-L$OPENBLAS_PATH"' >> /usr/src/myapp/.env

RUN /bin/bash -c "source /usr/src/myapp/.env; cargo install --path ."

CMD ["summaries_service"]