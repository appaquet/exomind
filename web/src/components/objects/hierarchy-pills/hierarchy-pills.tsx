import classNames from "classnames";
import React, { useState } from "react";
import { ICollection, } from "../../../store/collections";
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
        const list = this.props.collections.flatMap((collection) => {
            const hierarchy = getHierarchy(collection);
            if (hierarchy.length == 0) {
                return [];
            }

            return [<Pill key={collection.entityId} hierarchy={hierarchy} onClick={this.props.onCollectionClick} />];
        });
        if (list.length == 0) {
            return null;
        }

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

function Pill(props: { hierarchy: ICollection[], onClick?: (e: React.MouseEvent, col: ICollection) => void }) {
    const inner = props.hierarchy.map((col) => {
        const innerOnClick = (e: React.MouseEvent) => {
            props.onClick(e, col);
        };

        return (
            <li key={col.entityId} onClick={innerOnClick}>
                <span className="icon"><EntityIcon icon={col.icon} /></span>
                <span className="name">{col.name}</span>
            </li>
        );
    });

    const classes = classNames({
        pill: true,
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
        if (collection.entityId == 'favorites') {
            break;
        }

        out.push(collection);

        if (!collection.minParent) {
            break;
        }
        collection = collection.minParent;
    }

    return out.reverse();
}