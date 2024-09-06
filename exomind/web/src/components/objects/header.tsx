
import React, { MouseEvent } from 'react';
import EditableText from '../interaction/editable-text/editable-text';
import classNames from 'classnames';
import './header.less';
import { TraitIcon } from '../../utils/entities.js';
import EntityIcon from './entity-icon';
import { IStores, StoresContext } from '../../stores/stores';
import { IMenuItem } from '../layout/menu';
import _ from 'lodash';
import { IAction } from '../../utils/actions';

interface IProps {
    title: string;
    editableTitle?: string; // if title to be edited is different from displayed title (ex: emoji prefix)
    icon?: TraitIcon;
    actions: HeaderAction[];
    active: boolean;
    onTitleRename: (title: string) => void;
}

export class Header extends React.Component<IProps> {
    static contextType = StoresContext;
    declare context: IStores;

    render(): React.ReactNode {
        let rightActions;
        if (this.props.actions) {
            rightActions = <div className="right-actions">{this.renderActions()}</div>;
        }

        const titleClasses = classNames({
            title: true,
            active: this.props.active
        });

        return (
            <div className="header">
                <div className="icon">
                    {this.props.icon && <EntityIcon icon={this.props.icon} />}
                </div>

                <div className={titleClasses}>{this.renderTitle()}</div>

                {rightActions}
            </div>
        );
    }

    private renderTitle() {
        if (this.props.onTitleRename) {
            return <EditableText
                text={this.props.title}
                editText={this.props.editableTitle}
                doubleClick={true}
                onChange={this.handleTitleRename} />;
        } else {
            return this.props.title;
        }
    }

    private handleTitleRename = (newTitle: string) => {
        this.props.onTitleRename(newTitle);
    };

    private renderActions() {
        const visibleActions = this.props.actions.filter(action => !action.overflow);
        if (this.props.actions.length > visibleActions.length) {
            const showMenu = (e: MouseEvent) => {
                e.stopPropagation();
                this.context.session.showMenu({
                    items: _.chain(this.props.actions).sortBy((a) => a.order).map((a) => a.toMenuItem()).value(),
                }, e.currentTarget as HTMLElement);
            };

            visibleActions.push(new HeaderAction('More', 'ellipsis-v', showMenu, false));
        }

        const actionFragments: React.ReactNode[] = _.chain(visibleActions)
            .sortBy((a) => a.order)
            .map(action => {
                const classes = classNames({
                    'fa': true,
                    ['fa-' + action.icon]: true
                });
                return (
                    <li key={action.icon} onClick={(e) => action.callback(e)}>
                        <i className={classes} />
                    </li>
                );
            })
            .value();

        return <ul className="actions">{actionFragments}</ul>;
    }
}

export class HeaderAction {
    constructor(public label: string, public icon: string, public callback: (e: MouseEvent) => void, public overflow: boolean = false, public order: number = 0) {
    }

    static fromAction(action: IAction, overflow = false): HeaderAction {
        return new HeaderAction(action.label, action.icon, (e) => {
            action.execute(e);
        }, overflow, action.priority);
    }

    toMenuItem(): IMenuItem {
        return {
            label: this.label,
            icon: this.icon,
            onClick: (e) => this.callback(e)
        };
    }
}
