import { Exocore } from '.';
import * as protos from '../protos';

export class Registry {
    private _registeredMessages: { [name: string]: any } = {};

    registerMessage(message: any, fullName: string): any {
        message.prototype._proto_full_name = fullName;

        this._registeredMessages[fullName] = {
            fullName: fullName,
            message: message,
        }
    }

    messageFullName(message: any): string {
        let fullName = message._proto_full_name;
        if (!fullName && message.prototype) {
            fullName = message.prototype._proto_full_name;
        }

        const info = this._registeredMessages[fullName];
        if (!info) {
            console.error('Tried to get full name for an unregistered message', message);
            throw 'Tried to pack an unregistered message';
        }

        return info.fullName;
    }

    packToAny(message: any): protos.google.protobuf.IAny {
        const info = this._registeredMessages[message._proto_full_name];
        if (!info) {
            console.log('Tried to pack an unregistered message', message);
            throw 'Tried to pack an unregistered message';
        }

        return new protos.google.protobuf.Any({
            type_url: 'type.googleapis.com/' + info.fullName,
            value: info.message.encode(message).finish(),
        })
    }

    unpackAny(any: protos.google.protobuf.IAny): any {
        const fullName = this.canonicalFullName(any.type_url);
        const info = this._registeredMessages[fullName];
        if (!info) {
            console.error('Tried to unpack any any with unregistered type', fullName);
            throw 'Tried to pack an unregistered message';
        }

        return info.message.decode(any.value);
    }

    canonicalFullName(name: string) {
        return name.replace('type.googleapis.com/', '');
    }
}

export function matchTrait<T>(trait: protos.exocore.store.ITrait, matchMap: { [fullName: string]: (trait: protos.exocore.store.ITrait, message: any) => T }): T | null {
    const fullName = Exocore.registry.canonicalFullName(trait.message.type_url);

    if (fullName in matchMap) {
        const message = Exocore.registry.unpackAny(trait.message);
        return matchMap[fullName](trait, message);
    } else {
        return null;
    }
}