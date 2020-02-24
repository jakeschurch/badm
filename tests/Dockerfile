FROM instrumentisto/rust:nightly-slim
ENV RUST_BACKTRACE="full"

# create a new empty shell project
RUN mkdir /src && chmod -R a+w /src

# create non-root user
RUN useradd -ms /bin/bash ferris
USER ferris

# create a blank project
RUN USER=ferris cargo new --bin /src/badm
WORKDIR /src/badm

# Copy whole project
COPY . .

# "-Z unstable-options" is required for "--include-ignored"
CMD ["cargo", "test", "--", "-Z", "unstable-options", "--include-ignored", "--nocapture", "--test-threads=1"]
