import { action, makeObservable, observable } from 'mobx';
import { TraitIcon } from '../../utils/entities';
import { HeaderAction } from './header';

export class ContainerState {
    @observable title: string | ModifiableText;
    @observable icon: TraitIcon;
    @observable actions: HeaderAction[] = [];
    @observable closed: boolean;
    @observable showDetails: boolean;
    @observable active: boolean;

    constructor() {
        makeObservable(this);
    }

    @action pushHeaderAction(action: HeaderAction): void {
        this.actions.push(action);
    }

    @action prependHeaderAction(action: HeaderAction): void {
        this.actions.unshift(action);
    }

    @action addDetailsHeaderAction(): void {
        this.prependHeaderAction(new HeaderAction('Show details', 'info-circle', () => {
            this.toggleDetails();
        }));
    }

    @action toggleDetails(): void {
        this.showDetails = !this.showDetails;
    }
}

export class ModifiableText {
    constructor(public value: string, public onChange: (value: string) => void, public editValue?: string) {
    }

    toString(): string {
        return this.value;
    }
}
