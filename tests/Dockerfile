FROM rust

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

# ENV RUST_BACKTRACE 1

# "-Z unstable-options" is required for "--include-ignored"
CMD ["cargo", "test", "--", "--nocapture", "-Z", "unstable-options", "--include-ignored"]