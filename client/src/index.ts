import { w3cwebsocket as WebSocketClient } from "websocket";

const client = new WebSocketClient("ws://localhost:8000/ws");

client.onerror = error => {
    console.info("Connection failed", error);
};

client.onopen = () => {
    console.info("Connected");
    const sayHello = () => {
        if (client.readyState === client.OPEN) {
            client.send("Hello from client!");
        }
    };
    sayHello();
};

client.onclose = () => {
    console.info(`Connection closed`);
};

client.onmessage = message => {
    console.info("Received message", message);
};
