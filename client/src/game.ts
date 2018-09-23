export class Game {
    public allocateBuffer(size: number): VmBuffer {
        return new VmBuffer("");
    }

    public updatePlayer(world: World, player: PlayerId, input: Input): World {
        return world;
    }

    public updateWorld(world: World): World {
        return world;
    }

    public createPlayerId(id: number): PlayerId {
        return new PlayerId(id);
    }

    public addPlayer(id: PlayerId, world: World): World {
        return world;
    }

    public removePlayer(id: PlayerId, world: World): World {
        return world;
    }

    public deserializeInput(buf: VmBuffer): Input {
        return new Input(buf.getData());
    }

    public deserializeWorld(buf: VmBuffer): World {
        return new World();
    }

    public serializeInput(input: Input): VmBuffer {
        return new VmBuffer(input.data);
    }
}

export class Input {
    public data: string;

    constructor(data: string) {
        this.data = data;
    }

    public free() {}
}

export class PlayerId {
    private id: number;

    constructor(id: number) {
        this.id = id;
    }

    public free() {}

    public toNumber(): number {
        return this.id;
    }
}

export class VmBuffer {
    private data: string;

    constructor(data: string) {
        this.data = data;
    }

    public free() {}

    public putData(data: string) {
        this.data = data;
    }

    public getData(): string {
        return this.data;
    }
}

export class World {
    public free() {}
}
