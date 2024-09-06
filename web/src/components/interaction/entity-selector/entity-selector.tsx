import React from "react";
import { EntityTraits } from "../../../utils/entities";
import { Message } from "../../objects/message";
import Scrollable, { isVisibleWithinScrollable } from "../scrollable/scrollable";
import { IStores, StoresContext } from "../../../stores/stores";
import { observer } from "mobx-react";
import EntityIcon from "../../objects/entity-icon";
import { HierarchyPills } from "../../objects/hierarchy-pills/hierarchy-pills";
import './entity-selector.less';
import classNames from "classnames";
import { CancellableEvent } from "../../../utils/events";
import { ListenerToken, Shortcuts } from "../../../shortcuts";

interface IProps {
    multi?: boolean,
    entities: EntityTraits[],
    selectedIds?: string[],
    loading?: boolean,
    onSelect: (entity: EntityTraits, event?: CancellableEvent) => void,
    onUnselect?: (entity: EntityTraits, event?: CancellableEvent) => void,
    onNeedMore?: () => void,
    onBlur?: () => void,
}

interface IState {
    hoveredIndex?: number;
}

@observer
export class EntitySelector extends React.Component<IProps, IState> {
    static contextType = StoresContext;
    declare context: IStores;

    private shortcutToken: ListenerToken;

    constructor(props: IProps) {
        super(props);
        this.state = {};
        this.shortcutToken = Shortcuts.register([
            {
                key: 'n',
                callback: this.handleShortcutNext,
                disabledContexts: ['input'],
            },
            {
                key: 'ArrowDown',
                callback: this.handleShortcutNext,
                disabledContexts: [], // allow focusing out of search bar
            },
            {
                key: ['p', 'ArrowUp'],
                callback: this.handleShortcutPrevious,
                disabledContexts: ['input'],
            },
            {
                key: ['Mod-ArrowUp'],
                callback: this.handleShortcutTop,
                disabledContexts: ['input'],
            },
            {
                key: ['Mod-ArrowDown'],
                callback: this.handleShortcutBottom,
                disabledContexts: [],
            },
            {
                key: ['Space', 'Enter'],
                callback: this.handleShortcutSelect,
                disabledContexts: ['input'],
            },
            {
                key: ['Tab'],
                callback: this.handleShortcutBlur,
                disabledContexts: ['input'],
            },
        ]);
    }

    componentWillUnmount(): void {
        Shortcuts.unregister(this.shortcutToken);
    }

    render(): React.ReactNode {
        if (this.props.loading) {
            return <Message text="Loading..." showAfterMs={200} />;
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

                    e.stopPropagation();
                };

                return <li key={et.entity.id} id={`entity-${i}`} className={classes} onClick={(e) => handleClick(et, e)}>
                    {multi && <input type="checkbox" checked={checked} onChange={(e) => handleClick(et, e)} />}

                    <EntityIcon trait={priorityTrait} />

                    {priorityTrait.displayName}

                    {parents && <HierarchyPills collections={parents.get()} onCollectionClick={(e, col) => handleClick(col.entity, e)} />}
                </li>;
            });
    }

    private handleLoadMore = () => {
        this.props.onNeedMore?.();
    };

    private handleShortcutNext = (): boolean => {
        let idx = this.state.hoveredIndex ?? -1;
        idx += 1;

        if (idx >= this.props.entities.length) {
            idx = this.props.entities.length - 1;
        }

        this.hoverIndex(idx, 'down');
        return true;
    };

    private handleShortcutPrevious = (): boolean => {
        let idx = this.state.hoveredIndex ?? 0;
        idx -= 1;

        if (idx < 0) {
            if (this.props.onBlur) {
                this.setState({
                    hoveredIndex: undefined,
                });
                this.props.onBlur();
                return true;
            } else {
                idx = 0;
            }
        }

        this.hoverIndex(idx, 'up');
        return true;
    };

    private handleShortcutTop = (): boolean => {
        this.hoverIndex(0);
        return true;
    };

    private handleShortcutBottom = (): boolean => {
        this.hoverIndex(this.props.entities.length - 1);
        return true;
    };

    private hoverIndex(idx: number, dir: 'up' | 'down' | null = null) {
        if (idx >= this.props.entities.length - 10) {
            this.props.onNeedMore?.();
        }

        let el = document.getElementById(`entity-${idx}`);
        if (el && !isVisibleWithinScrollable(el)) {
            if (dir == 'up') {
                const scrollIdx = Math.max(idx - 3, 0);
                el = document.getElementById(`entity-${scrollIdx}`);
            } else if (dir == 'down') {
                const scrollIdx = Math.min(idx - 2, this.props.entities.length - 1);
                el = document.getElementById(`entity-${scrollIdx}`);
            }
            el?.scrollIntoView({ behavior: 'smooth' });
        }

        this.setState({
            hoveredIndex: idx,
        });
    }

    private handleShortcutSelect = (): boolean => {
        if (this.state.hoveredIndex == undefined) {
            return false;
        }

        const entity = this.props.entities[this.state.hoveredIndex ?? 0];
        const selectedIds = new Set(this.props.selectedIds ?? []);

        this.hoverIndex(0);

        if (!selectedIds.has(entity.id)) {
            this.props.onSelect(entity, null);
        } else {
            this.props.onUnselect(entity, null);
        }

        return true;
    };

    private handleShortcutBlur = (): boolean => {
        if (this.state.hoveredIndex == undefined || !this.props.onBlur) {
            return false;
        }
        this.setState({
            hoveredIndex: undefined,
        });
        this.props.onBlur();
        return true;
    };
}