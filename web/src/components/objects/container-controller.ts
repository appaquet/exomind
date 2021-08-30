import { makeObservable, observable } from 'mobx';
import { TraitIcon } from '../../utils/entities';
import { HeaderAction } from './header';

export class ContainerController {
    @observable title: string | ModifiableText;
    @observable icon: TraitIcon;
    @observable actions: HeaderAction[];
    @observable closed: boolean;
    @observable details: boolean;

    constructor() {
        makeObservable(this);
    }
}

export class ModifiableText {
    constructor(public value: string, public onChange: (value: string) => void, public editValue?: string) {
    }

    toString(): string {
        return this.value;
    }
}
