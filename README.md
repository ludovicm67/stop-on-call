# Stop On Call

Stop On Call is a simple HTTP endpoint that stops the server when called.

## Environment Variables

| Variable Name           | Default Value | Description                                       |
| ----------------------- | ------------- | ------------------------------------------------- |
| `STOP_ON_CALL_HOSTNAME` | `0.0.0.0`     | The hostname the server will bind to.             |
| `STOP_ON_CALL_PORT`     | `8080`        | The port the server will listen on.               |
| `STOP_ON_CALL_METHOD`   | `GET`         | The HTTP method to trigger the stop.              |
| `STOP_ON_CALL_SECRET`   |               | The secret required to stop the server. Optional. |
