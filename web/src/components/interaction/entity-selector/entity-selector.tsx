import React, { SyntheticEvent } from "react";
import { EntityTraits } from "../../../utils/entities";
import { Message } from "../../objects/message";
import Scrollable from "../scrollable/scrollable";
import _ from 'lodash';
import { IStores, StoresContext } from "../../../stores/stores";
import { observer } from "mobx-react";
import EntityIcon from "../../objects/entity-icon";
import { HierarchyPills } from "../../objects/hierarchy-pills/hierarchy-pills";
import './entity-selector.less'

interface IProps {
    multi?: boolean,
    entities: EntityTraits[],
    selectedIds?: string[],
    loading?: boolean,
    onSelect: (entity: EntityTraits, event: SyntheticEvent) => void,
    onUnselect?: (entity: EntityTraits, event: SyntheticEvent) => void,
    onNeedMore?: () => void,
}

// TODO: Keyboard up & down

@observer
export class EntitySelector extends React.Component<IProps, unknown> {
    static contextType = StoresContext;
    declare context: IStores;

    constructor(props: IProps) {
        super(props);
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

    private renderEntities(): React.ReactNode {
        const selectedIds = new Set(this.props.selectedIds ?? []);
        const multi = this.props.multi ?? true;

        return _.chain(this.props.entities)
            .uniqBy(et => et.id)
            .map((et) => {
                const priorityTrait = et.priorityTrait; //et.traitOfType<exomind.base.v1.ICollection>(exomind.base.v1.Collection);
                if (!priorityTrait) {
                    return null;
                }

                const parents = this.context.collections.getEntityParents(et);
                const checked = selectedIds.has(et.id);

                const handleClick = (entity: EntityTraits, e: SyntheticEvent) => {
                    if (checked) {
                        this.props.onUnselect?.(entity, e);
                    } else {
                        this.props.onSelect(entity, e);
                    }

                    e.stopPropagation();
                };

                return <li key={et.entity.id} onClick={(e) => handleClick(et, e)}>
                    {multi && <input type="checkbox" checked={checked} onChange={(e) => handleClick(et, e)} />}

                    <EntityIcon trait={priorityTrait} />

                    {priorityTrait.displayName}

                    {parents && <HierarchyPills collections={parents.get()} onCollectionClick={(e, col) => handleClick(col.entity, e)} />}
                </li>
            })
            .value();
    }

    private handleLoadMore = () => {
        this.props.onNeedMore?.();
    }
}