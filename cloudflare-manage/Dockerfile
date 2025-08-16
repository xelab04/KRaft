FROM registry.opensuse.org/opensuse/bci/python:3.11

WORKDIR /app

RUN pip install --upgrade pip

RUN adduser -D nonroot

COPY --chown=nonroot:nonroot . .

EXPOSE 5000

CMD ["python", "main.py"]
