import classNames from 'classnames';
import { Exocore, QueryBuilder, WatchedQueryWrapper } from 'exocore';
import React, { ChangeEvent, KeyboardEvent } from 'react';
import { ExpandableQuery } from '../../../stores/queries';
import Debouncer from '../../../utils/debouncer';
import { EntityTraits } from '../../../utils/entities';
import { EntitySelector } from '../../interaction/entity-selector/entity-selector';
import { SelectedLink } from '../../interaction/html-editor/html-editor';
import EntityIcon from '../entity-icon';
import './link-selector.less';

interface IProps {
    initialValue?: string;
    onDone: (link: SelectedLink) => void;
    onCancel: () => void;
}

interface IState {
    inputValue: string;
    entity?: EntityTraits;
    entities?: EntityTraits[];
}

export default class LinkSelector extends React.Component<IProps, IState> {
    private inputRef: React.RefObject<HTMLInputElement> = React.createRef();

    private searchDebouncer: Debouncer;

    private searchEntityQuery?: ExpandableQuery;
    private searchEntityText?: string;

    private entityQuery?: WatchedQueryWrapper;
    private entityQueryId?: string;

    constructor(props: IProps) {
        super(props);

        this.searchDebouncer = new Debouncer(200);

        this.state = {
            inputValue: props.initialValue ?? ''
        };
    }

    componentDidMount(): void {
        this.inputRef.current?.focus();
    }

    componentWillUnmount(): void {
        this.entityQuery?.free();
        this.searchEntityQuery?.free();
    }

    render(): React.ReactNode {
        const classes = classNames({
            'note-link-selector': true,
        });

        const isEntityLink = this.state.inputValue.startsWith('entity://');
        const renderEntitySelector = this.state.inputValue.length >= 3 && !this.state.inputValue.startsWith('htt');
        return (
            <div className={classes}>
                <div className="text">Enter a link or type entity name</div>

                {!isEntityLink && <div className="value">
                    <input type="text"
                        ref={this.inputRef}
                        value={this.state.inputValue}
                        onChange={this.onValueChange}
                        onKeyDown={this.onKeyDown}
                    />
                </div>}

                {isEntityLink && this.renderEntity()}

                {!isEntityLink && renderEntitySelector && this.renderEntitySelector()}

                <div className="buttons">
                    <button onClick={this.onCancel}>Cancel</button>
                    <button onClick={this.onClear}>Clear</button>
                    <button onClick={this.onDone}>Done</button>
                </div>
            </div>
        );
    }

    private renderEntity(): React.ReactNode {
        const entityId = this.state.inputValue.replace('entity://', '');

        if (entityId != this.entityQueryId) {
            this.entityQuery?.free();

            const entityQuery = QueryBuilder.withIds(entityId).build();
            this.entityQuery = Exocore.store
                .watchedQuery(entityQuery)
                .onChange((res) => {
                    const entity = new EntityTraits(res.entities[0].entity);
                    this.setState({
                        entity: entity,
                    });
                });

            this.entityQueryId = entityId;

            return <div></div>;
        }

        const handleOnClick = () => {
            this.setState({ inputValue: '' })
            setTimeout(() => {
                this.inputRef.current?.focus();
            });
        };

        if (this.state.entity && this.state.entity.priorityTrait) {
            const et = this.state.entity.priorityTrait;
            return (
                <div className="entity" onClick={handleOnClick}>
                    <span className="icon"><EntityIcon icon={et.icon} /></span>
                    <span className="name">{et.displayName}</span>
                </div>
            );
        }
    }

    private renderEntitySelector(): React.ReactNode {
        return <EntitySelector
            multi={false}
            entities={this.state.entities ?? []}
            onSelect={this.handleEntitySelect}
        />;
    }

    private handleEntitySelect = (entity: EntityTraits): void => {
        this.props.onDone({
            url: `entity://${entity.id}`,
            title: entity.priorityTrait?.displayName
        });
    }

    private onValueChange = (e: ChangeEvent<HTMLInputElement>): void => {
        this.searchDebouncer.debounce(() => {
            const queryText = e.target.value;
            if (queryText != this.searchEntityText) {
                this.searchEntityQuery?.free();
                this.searchEntityText = queryText;

                const query = QueryBuilder
                    .matches(queryText)
                    .count(30)
                    .build();
                this.searchEntityQuery = new ExpandableQuery(query, () => {
                    const entities: EntityTraits[] = [];
                    for (const res of this.searchEntityQuery.results()) {
                        entities.push(new EntityTraits(res.entity));
                    }
                    this.setState({
                        entities
                    });
                })
            }
        });

        this.setState({
            inputValue: e.target.value
        });
    }

    private onKeyDown = (e: KeyboardEvent): void => {
        if (e.key == 'Enter') {
            this.props.onDone({ url: this.state.inputValue });
        } else if (e.key == 'ArrowUp' || e.key == 'ArrowDown') {
            this.inputRef.current?.blur();
        }
    }

    private onCancel = (): void => {
        this.props.onCancel();
    }

    private onClear = (): void => {
        this.props.onDone(null);
    }

    private onDone = (): void => {
        this.props.onDone({ url: this.state.inputValue });
    }
}