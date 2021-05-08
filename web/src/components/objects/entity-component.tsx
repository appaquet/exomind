import { Exocore, exocore, MutationBuilder, QueryBuilder, toProtoTimestamp, WatchedQueryWrapper } from 'exocore';
import React from 'react';
import { exomind } from '../../protos';
import { EntityTrait, EntityTraits } from "../../utils/entities";
import { ModalStore } from "../../stores/modal-store";
import { CollectionSelector } from '../modals/collection-selector/collection-selector';
import TimeSelector from '../modals/time-selector/time-selector';
import { ContainerController, ModifiableText } from "./container-controller";
import './entity-component.less';
import { Selection } from "./entity-list/selection";
import { HeaderAction } from "./header";
import { Message } from "./message";

const Task = React.lazy(() => import(/*webpackChunkName: "component-task"*/'./task/task'));
const Note = React.lazy(() => import(/*webpackChunkName: "component-note"*/'./note/note'));
const Collection = React.lazy(() => import(/*webpackChunkName: "component-collection"*/'./collection/collection'));
const Link = React.lazy(() => import(/*webpackChunkName: "component-link"*/'./link/link'));
const Email = React.lazy(() => import(/*webpackChunkName: "component-email"*/'./email/email'));
const EmailThread = React.lazy(() => import(/*webpackChunkName: "component-email-thread"*/'./email/email-thread'));
const DraftEmail = React.lazy(() => import(/*webpackChunkName: "component-draft-email"*/'./draft-email/draft-email'));

interface Props {
    entityId: string;
    traitId?: string;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerController?: ContainerController;
}

interface State {
    results?: exocore.store.EntityResults;
    entityTraits?: EntityTraits;
    trait?: EntityTrait<unknown>;
}

export class EntityComponent extends React.Component<Props, State> {
    private entityQuery: WatchedQueryWrapper;

    constructor(props: Props) {
        super(props);

        const query = QueryBuilder.withIds(props.entityId).build();
        this.entityQuery = Exocore.store
            .watchedQuery(query)
            .onChange(this.handleNewResults);

        if (props.containerController) {
            props.containerController.actions = [
                new HeaderAction('clock-o', this.handleShowTimeSelector),
                new HeaderAction('folder-open-o', this.handleShowCollectionSelector)
            ];
        }

        this.state = {};
    }

    render(): React.ReactNode {
        if (this.state.results && this.state.results.entities.length > 0) {
            if (!this.state.trait) {
                return <Message text="Trait not found" />;
            }

            const inner = this.state.trait.match({
                collection: (col) => {
                    return <Collection
                        entity={this.state.entityTraits}
                        collection={col}
                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                    />;
                },
                note: (note) => {
                    return <Note
                        entity={this.state.entityTraits}
                        noteTrait={note}
                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                    />;
                },
                task: (task) => {
                    return <Task
                        entity={this.state.entityTraits}
                        taskTrait={task}
                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                    />;
                },
                link: (link) => {
                    return <Link
                        entity={this.state.entityTraits}
                        linkTrait={link}
                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                    />;
                },
                emailThread: () => {
                    return <EmailThread
                        entity={this.state.entityTraits}
                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                        containerController={this.props.containerController}
                    />;
                },
                email: (email) => {
                    return <Email
                        entity={this.state.entityTraits}
                        emailTrait={email}
                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                    />;
                },
                draftEmail: (draft) => {
                    return <DraftEmail
                        entity={this.state.entityTraits}
                        draftTrait={draft}
                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                        containerController={this.props.containerController}
                    />;
                },
                default: () => {
                    return <Message text="Unsupported entity" />;
                },
            });

            const loading = <Message key={'loading'} text="Loading..." showAfterMs={200} />;
            return <React.Suspense fallback={loading}>{inner}</React.Suspense>

        } else if (this.state.results) {
            return <Message key={'notfound'} text="Not found" />;

        } else {
            return <Message key={'loading'} text="Loading..." showAfterMs={200} />;
        }
    }

    private handleNewResults = (results: exocore.store.EntityResults): void => {
        if (results && results.entities.length > 0) {
            const et = new EntityTraits(results.entities[0].entity)

            let trait: EntityTrait<unknown>;
            if (this.props.traitId) {
                trait = et.trait(this.props.traitId);
            } else {
                trait = et.priorityTrait;
            }

            this.props.containerController.icon = trait.icon;
            if (trait.canEditName) {
                this.props.containerController.title = new ModifiableText(trait.displayName, (newTitle: string) => {
                    trait.rename(newTitle);
                }, trait.editableName);
            } else {
                this.props.containerController.title = trait.displayName;
            }

            this.setState({
                results: results,
                entityTraits: et,
                trait: trait,
            });

        } else {
            this.setState({
                results: undefined,
                entityTraits: undefined,
                trait: undefined,
            });
        }

        this.setState({ results });
    }


    private handleShowCollectionSelector = (): void => {
        if (this.state.results && this.state.results.entities.length > 0) {
            const entity = new EntityTraits(this.state.results.entities[0].entity);
            ModalStore.showModal(() => {
                return <CollectionSelector entity={entity} />;
            });
        }
    }

    private handleShowTimeSelector = (): void => {
        ModalStore.showModal(() => {
            return <TimeSelector onSelectionDone={(date) => this.handleCloseTimeSelector(date)} />;
        });
    }

    private handleCloseTimeSelector(date: Date): void {
        ModalStore.hideModal();

        let mb = MutationBuilder
            .updateEntity(this.state.entityTraits.id)
            .putTrait(new exomind.base.Snoozed({
                untilDate: toProtoTimestamp(date),
            }), "snoozed")
            .returnEntities();

        const parentRelation = this.state.entityTraits
            .traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild)
            .find((trt) => trt.message.collection.entityId == 'inbox')
        if (parentRelation) {
            mb = mb.deleteTrait(parentRelation.id);
        }

        Exocore.store.mutate(mb.build());
    }

    componentWillUnmount(): void {
        this.entityQuery.free();
    }
}
