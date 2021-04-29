import classNames from "classnames";
import React, { useState } from "react";
import { ICollection } from "../../../store/collections";
import EntityIcon from "../entity-icon";
import './hierarchy-pills.less';

export interface IProps {
    collections: ICollection[],
    onCollectionClick?: (e: React.MouseEvent, collection: ICollection) => void,
}

export class HierarchyPills extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props)
    }

    render(): React.ReactNode {
        const list = this.props.collections.map((collection) => {
            return <Pill key={collection.entityId} collection={collection} onClick={this.props.onCollectionClick} />;
        });

        const classes = classNames({
            'hierarchy-pills': true,
        });

        return (
            <div className={classes}>
                <ul>{list}</ul>
            </div>
        );
    }
}

function Pill(props: { collection: ICollection, onClick?: (e: React.MouseEvent, col: ICollection)=>void }) {
    const [hovered, setHovered] = useState(false);
    const hierarchy = getHierarchy(props.collection);

    const inner = hierarchy.map((col) => {
        const innerOnClick = (e: React.MouseEvent) => {
            props.onClick(e, col);
        };

        return (
            <li key={col.entityId} onMouseOver={() => setHovered(true)} onMouseLeave={() => setHovered(false)} onClick={innerOnClick}>
                <EntityIcon icon={col.icon} />
                <span className="name">{col.name}</span>
            </li>
        );
    });

    const classes = classNames({
        hovered: hovered,
        clickable: !!props.onClick,
    })

    return (
        <li className={classes}>
            <ul>
                {inner}
            </ul>
        </li>
    );
}

function getHierarchy(collection: ICollection) {
    const out = [];

    while (collection != null) {
        out.push(collection);

        const parents = collection.parents ?? [];
        if (parents.length > 0) {
            collection = collection.parents[0];
        } else {
            break;
        }
    }

    return out.reverse();
}