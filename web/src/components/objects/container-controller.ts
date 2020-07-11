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

    set icon(value: string) {
        this.set('icon', value);
    }

    get icon(): string {
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
    value: string;
    onChange: (value: string) => void;

    constructor(value: string, onChange: (value: string) => void) {
        this.value = value;
        this.onChange = onChange;
    }

    toString(): string {
        return this.value;
    }
}
