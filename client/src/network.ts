import { w3cwebsocket as WebSocketClient } from "websocket";

type ServerMessage = WorldStateMessage | PlayerInputMessage;

export interface WorldStateMessage {
    localPlayerId: number;
    frame: number;
    world: Uint8Array;
}

export interface PlayerInputs {
    [id: string]: Uint8Array;
}

export interface PlayerInputMessage {
    newPlayers: number[];
    removedPlayers: number[];
    inputs: PlayerInputs;
}

export interface LocalPlayerInput {
    frame: number;
    input: Uint8Array;
}

export class NetworkHandler {
    public onWorldState: (world: WorldStateMessage) => void;
    public onPlayerInputs: (inputs: PlayerInputMessage) => void;
    private client: WebSocketClient;
    private pendingInputs: PlayerInputMessage[];
    private receivedWorldState: boolean;

    constructor() {
        this.client = new WebSocketClient("ws://" + location.host + "/ws");
        this.pendingInputs = [];
        this.receivedWorldState = false;
        this.onWorldState = _ => {};
        this.onPlayerInputs = _ => {};
        this.client.onopen = () => this.onOpen();
        this.client.onerror = err => this.error(err);
        this.client.onclose = () => this.onClose();
        this.client.onmessage = msg => this.onMessage(msg);
    }

    public sendInput(input: LocalPlayerInput) {
        this.client.send(JSON.stringify({
            input: {
                frame: input.frame,
                input: [].slice.call(input.input),
            },
        }));
    }

    public joinGame(frame: number) {
        console.info("Joining on frame:", frame);
        this.client.send(JSON.stringify({ join: { frame } }));
    }

    private error(err: Error) {
        console.error(`Connection error: ${err}`);
    }

    private onOpen() {
        console.info("Connected");
    }

    private onClose() {
        console.info("Disconnected");
    }

    private onMessage(message: any) {
        console.debug("Received message:", message);
        const payload = parseMessagePayload(message.data);
        if (isWorldState(payload)) {
            this.receivedWorldState = true;
            this.onWorldState(payload);
            this.pendingInputs.forEach(this.onPlayerInputs);
            this.pendingInputs = [];
        } else {
            if (this.receivedWorldState) {
                this.onPlayerInputs(payload);
            } else {
                this.pendingInputs.push(payload);
            }
        }
    }
}

function parseMessagePayload(message: any): ServerMessage {
    const msg = JSON.parse(message);
    if (msg.world !== undefined) {
        return {
            frame: msg.frame,
            localPlayerId: msg.localPlayerId,
            world: new Uint8Array(msg.world),
        };
    } else {
        const inputs: PlayerInputs = {};
        for (const key of Object.keys(msg.inputs)) {
            inputs[key] = new Uint8Array(msg.inputs[key]);
        }
        return {
            frame: msg.frame,
            inputs,
            newPlayers: msg.newPlayers,
            removedPlayers: msg.removedPlayers,
        };
    }
}

function isWorldState(message: ServerMessage): message is WorldStateMessage {
    const m = message as WorldStateMessage;
    return m.frame !== undefined && m.world !== undefined && m.localPlayerId !== undefined;
}
