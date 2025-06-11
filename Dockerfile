FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY app/ ./app/

# default port
ENV PORT=8463
EXPOSE 8463

ENTRYPOINT ["sh", "-c"]
CMD ["uvicorn app.main:app --host 0.0.0.0 --port $PORT --loop uvloop --workers 1"]
