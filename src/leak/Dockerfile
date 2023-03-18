FROM gcc:12 as builder
WORKDIR /app
COPY ./main.c ./
RUN gcc -O0 -static -o leak.out main.c

FROM gcr.io/distroless/cc
COPY --from=builder /app/leak.out /app/leak.out
WORKDIR /app
CMD ["./leak.out"] 
