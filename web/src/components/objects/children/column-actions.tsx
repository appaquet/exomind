import { Exocore, exocore, MutationBuilder, QueryBuilder } from 'exocore';
import React from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../utils/entities';
import InputModal from '../../modals/input-modal/input-modal';
import { Selection } from "../entity-list/selection";
import { IStores, StoresContext } from '../../../stores/stores';
import { ListenerToken, Shortcuts } from '../../../shortcuts';
import { Actions, IAction } from '../../../utils/actions';
import { CancellableEvent } from '../../../utils/events';
import classNames from 'classnames';
import "./column-actions.less";

interface IProps {
    parent: EntityTraits;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    onCreated: (et: EntityTraits) => void;

    removeOnPostpone?: boolean;
}

interface IState {
    selectionActions: IAction[];
}

export class ColumnActions extends React.Component<IProps, IState> {
    private shortcutToken: ListenerToken;
    private mounted = true;

    static contextType = StoresContext;
    declare context: IStores;

    constructor(props: IProps) {
        super(props);

        this.shortcutToken = Shortcuts.register([
            {
                key: 'e',
                callback: (e) => {
                    if (this.state.selectionActions.length > 0) {
                        this.handleExecuteAction(e, this.state.selectionActions.find((a) => a.key == 'remove-from-parent'));
                        return true;
                    } else {
                        return false;
                    }
                },
                disabledContexts: ['input', 'modal'],
            },
            {
                key: 'z',
                callback: (e) => {
                    if (this.state.selectionActions.length > 0) {
                        this.handleExecuteAction(e, this.state.selectionActions.find((a) => a.key == 'snooze'));
                        return true;
                    } else {
                        return false;
                    }
                },
                disabledContexts: ['input', 'modal'],
            }
        ]);

        this.state = {
            selectionActions: [],
        }

        this.fetchSelectedEntities();
    }

    componentWillUnmount() {
        Shortcuts.unregister(this.shortcutToken);
        this.mounted = false;
    }

    componentDidUpdate(): void {
        this.fetchSelectedEntities();
    }

    render(): React.ReactNode {
        if (this.props.selection && this.props.selection.isMulti && this.state.selectionActions.length > 0) {
            return this.renderSelectionActions();
        } else {
            return this.renderCreationActions();
        }
    }

    private renderCreationActions(): React.ReactNode {
        return <div className="column-bottom-actions">
            <ul>
                <li onClick={this.handleNewNoteClick}><i className="new-note" /></li>
                <li onClick={this.handleNewCollectionClick}><i className="new-collection" /></li>
                <li onClick={this.handleNewTaskClick}><i className="new-task" /></li>
                <li onClick={this.handleNewLinkClick}><i className="new-link" /></li>
            </ul>
        </div>
    }

    private renderSelectionActions(): React.ReactNode {
        return <div className="column-bottom-actions">
            <ul>
                {this.state.selectionActions.map((action) => {
                    return <li key={action.label} onClick={(e) => this.handleExecuteAction(e, action)}>
                        <i className={classNames({
                            'fa': true,
                            ['fa-' + action.icon]: true,
                        })} />
                    </li>
                })}
            </ul>
        </div>
    }

    private handleNewNoteClick = () => {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Note({
                title: 'New note',
            }))
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewCollectionClick = () => {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Collection({
                name: 'New collection',
            }))
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewTaskClick = () => {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Task())
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewLinkClick = () => {
        const createLink = (url?: string) => {
            this.context.session.hideModal();

            if (!url) {
                return;
            }

            const mutation = MutationBuilder
                .createEntity()
                .putTrait(new exomind.base.v1.Link({
                    url: url,
                }))
                .putTrait(new exomind.base.v1.CollectionChild({
                    collection: new exocore.store.Reference({
                        entityId: this.props.parent.id,
                    }),
                    weight: new Date().getTime(),
                }), `child_${this.props.parent.id}`)
                .returnEntities()
                .build();

            this.executeNewEntityMutation(mutation);
        }

        this.context.session.showModal(() => {
            return <InputModal
                text="URL of the link"
                onDone={createLink} />;
        });
    }

    private executeNewEntityMutation(mutation: exocore.store.MutationRequest) {
        Exocore.store.mutate(mutation).then(result => {
            if (result.entities.length > 0 && this.props.onCreated) {
                this.props.onCreated(new EntityTraits(result.entities[0]));
            }
        });
    }

    private async fetchSelectedEntities() {
        const ids = (this.props.selection?.items ?? [])
            .map((sel) => sel.entityId)
            .filter((et) => !!et);

        const results = await Exocore.store.query(QueryBuilder.withIds(ids).build());
        const entities = results.entities.map((res) => new EntityTraits(res.entity));

        const actions = Actions.forSelectedEntities(entities, { parent: this.props.parent });

        if (this.mounted) {
            this.setState({ selectionActions: actions });
        }
    }

    private async handleExecuteAction(e: CancellableEvent, action: IAction | null): Promise<boolean> {
        if (!action) {
            return false;
        }

        const res = await action.execute(e, action);
        if (res == 'remove') {
            if (this.props.selection) {
                this.props.onSelectionChange(this.props.selection.cleared());
            }
        }

        return true;
    }
}

