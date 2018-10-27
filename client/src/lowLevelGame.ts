export class Handle<Kind extends string> {
    kind: Kind;
    readonly value: number;

    constructor(value: number, kind: Kind) {
        this.kind = kind;
        this.value = value;
    }
}

export type WorldHandle = Handle<"world">;
export type InputHandle = Handle<"input">;
export type BufferHandle = Handle<"buffer">;

export class LowLevelGame {
    private readonly instance: WebAssembly.Instance;

    public constructor(instance: WebAssembly.Instance) {
        this.instance = instance;
    }

    public initialWorld(): WorldHandle {
        let value = this.instance.exports.initial_world();
        return new Handle(value, "world");
    }

    public updateWorld(world: WorldHandle): WorldHandle {
        let value = this.instance.exports.update_world(world.value);
        return new Handle(value, "world");
    }

    public updatePlayer(world: WorldHandle, playerId: number, input: InputHandle): WorldHandle {
        let value = this.instance.exports.update_player(world.value, playerId, input.value);
        return new Handle(value, "world");
    }

    public addPlayer(world: WorldHandle, playerId: number): WorldHandle {
        let value = this.instance.exports.add_player(world.value, playerId);
        return new Handle(value, "world");
    }

    public removePlayer(world: WorldHandle, playerId: number): WorldHandle {
        let value = this.instance.exports.remove_player(world.value, playerId);
        return new Handle(value, "world");
    }

    public allocateBuffer(size: number): BufferHandle {
        let value = this.instance.exports.allocate_buffer(size);
        return new Handle(value, "buffer");
    }

    public freeHandle<Kind extends string>(handle: Handle<Kind>) {
        this.instance.exports.free_handle(handle.value);
    }

    public bufferPtr(buffer: BufferHandle): number {
        return this.instance.exports.buffer_ptr(buffer.value);
    }

    public bufferSize(buffer: BufferHandle): number {
        return this.instance.exports.buffer_size(buffer.value);
    }

    public deserializeWorld(buffer: BufferHandle): WorldHandle {
        let value = this.instance.exports.deserialize_world(buffer.value);
        return new Handle(value, "world");
    }

    public deserializeInput(buffer: BufferHandle): InputHandle {
        let value = this.instance.exports.deserialize_input(buffer.value);
        return new Handle(value, "input");
    }

    public serializeInput(input: InputHandle): BufferHandle {
        let value = this.instance.exports.serialize_input(input.value);
        return new Handle(value, "buffer");
    }

    public readMemory(ptr: number, size: number): Uint8Array {
        return this.memory().slice(ptr, ptr + size);
    }

    public writeMemory(ptr: number, data: Uint8Array) {
        this.memory().set(data, ptr);
    }

    private memory(): Uint8Array {
        let memory = this.instance.exports.memory as WebAssembly.Memory;
        return new Uint8Array(memory.buffer);
    }
}
