import { TraitIcon } from '../../store/entities';
import ObservableDictionary from '../../utils/observable-dictionary';
import { HeaderAction } from './header';

// TODO: Move to mobx observable
export class ContainerController extends ObservableDictionary {
    set title(value: string | ModifiableText) {
        this.set('title', value);
    }

    get title(): string | ModifiableText {
        return this.get('title');
    }

    set icon(value: TraitIcon) {
        this.set('icon', value);
    }

    get icon(): TraitIcon {
        return this.get('icon');
    }

    set actions(value: HeaderAction[]) {
        this.set('actions', value);
    }

    get actions(): HeaderAction[] {
        return this.get('actions');
    }

    get closed(): boolean {
        return this.get('closed');
    }

    close(): void {
        this.set('closed', true);
    }
}

export class ModifiableText {
    constructor(public value: string, public onChange: (value: string) => void, public editValue?: string) {
    }

    toString(): string {
        return this.value;
    }
}
