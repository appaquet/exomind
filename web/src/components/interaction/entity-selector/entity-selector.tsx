import React from "react";
import { EntityTraits } from "../../../utils/entities";
import { Message } from "../../objects/message";
import Scrollable from "../scrollable/scrollable";
import { IStores, StoresContext } from "../../../stores/stores";
import { observer } from "mobx-react";
import EntityIcon from "../../objects/entity-icon";
import { HierarchyPills } from "../../objects/hierarchy-pills/hierarchy-pills";
import './entity-selector.less'
import classNames from "classnames";
import { CancellableEvent } from "../../../utils/events";

interface IProps {
    multi?: boolean,
    entities: EntityTraits[],
    selectedIds?: string[],
    loading?: boolean,
    onSelect: (entity: EntityTraits, event: CancellableEvent) => void,
    onUnselect?: (entity: EntityTraits, event: CancellableEvent) => void,
    onNeedMore?: () => void,
}

interface IState {
    hoveredIndex?: number;
}

@observer
export class EntitySelector extends React.Component<IProps, IState> {
    static contextType = StoresContext;
    declare context: IStores;

    constructor(props: IProps) {
        super(props);

        this.state = {};
    }

    render(): React.ReactNode {
        if (this.props.loading) {
            return <Message text="Loading..." showAfterMs={200} />
        }

        return (
            <div className="entity-selector">
                <Scrollable loadMoreItems={15} onNeedMore={this.handleLoadMore} nbItems={this.props.entities.length}>
                    <ul>
                        {this.renderEntities()}
                    </ul>
                </Scrollable>
            </div>
        );
    }

    componentDidMount(): void {
        document.addEventListener('keydown', this.handleKeyDown, false);
    }

    componentWillUnmount(): void {
        document.removeEventListener('keydown', this.handleKeyDown, false);
    }

    private renderEntities(): React.ReactNode {
        const selectedIds = new Set(this.props.selectedIds ?? []);
        const multi = this.props.multi ?? true;

        return this.props.entities
            .map((et, i) => {
                const priorityTrait = et.priorityTrait;
                if (!priorityTrait) {
                    return null;
                }

                const parents = this.context.collections.getEntityParents(et);
                const checked = selectedIds.has(et.id);
                const classes = classNames({
                    hovered: this.state.hoveredIndex === i,
                });

                const handleClick = (entity: EntityTraits, e: CancellableEvent) => {
                    if (checked) {
                        this.props.onUnselect?.(entity, e);
                    } else {
                        this.props.onSelect(entity, e);
                    }

                    e?.stopPropagation();
                };

                return <li key={et.entity.id} id={`entity-${i}`} className={classes} onClick={(e) => handleClick(et, e)}>
                    {multi && <input type="checkbox" checked={checked} onChange={(e) => handleClick(et, e)} />}

                    <EntityIcon trait={priorityTrait} />

                    {priorityTrait.displayName}

                    {parents && <HierarchyPills collections={parents.get()} onCollectionClick={(e, col) => handleClick(col.entity, e)} />}
                </li>
            });
    }

    private handleLoadMore = () => {
        this.props.onNeedMore?.();
    }

    private handleKeyDown = (e: KeyboardEvent): void => {
        if (e.key == 'ArrowUp') {
            let idx = this.state.hoveredIndex ?? 0;
            idx -= 1;
            if (idx < 0) {
                idx = 0;
            }

            document.getElementById(`entity-${idx}`)?.scrollIntoView();
            this.setState({
                hoveredIndex: idx,
            });
            e.preventDefault();
            e.stopPropagation();

        } else if (e.key == 'ArrowDown') {
            let idx = this.state.hoveredIndex ?? -1;
            idx += 1;
            if (idx >= this.props.entities.length) {
                idx = this.props.entities.length - 1;
            } else if (idx >= this.props.entities.length - 10) {
                this.props.onNeedMore?.();
            }

            document.getElementById(`entity-${idx}`)?.scrollIntoView();
            this.setState({
                hoveredIndex: idx,
            });
            e.preventDefault();
            e.stopPropagation();

        } else if (e.key == 'Enter' || e.key == ' ') {
            const entity = this.props.entities[this.state.hoveredIndex ?? 0];
            const selectedIds = new Set(this.props.selectedIds ?? []);

            if (!selectedIds.has(entity.id)) {
                this.props.onSelect(entity, null);
            } else {
                this.props.onUnselect(entity, null);
            }

            e.preventDefault();
            e.stopPropagation();
        }
    }
}