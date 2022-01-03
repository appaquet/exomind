
import React, { MouseEvent } from 'react';
import EditableText from '../interaction/editable-text/editable-text';
import classNames from 'classnames';
import './header.less';
import { TraitIcon } from '../../utils/entities.js';
import EntityIcon from './entity-icon';
import { IStores, StoresContext } from '../../stores/stores';
import { IMenuItem } from '../layout/menu';

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
    }

    private renderActions() {
        const actionFragments: React.ReactFragment[] = this.props.actions
            .filter((a) => !a.overflow)
            .map(action => {
                const classes = classNames({
                    'fa': true,
                    ['fa-' + action.icon]: true
                });
                return (
                    <li key={action.icon} onClick={() => this.handleActionClick(action)}>
                        <i className={classes} />
                    </li>
                );
            });

        if (this.props.actions.length > actionFragments.length) {
            const showMenu = (e: MouseEvent) => {
                e.stopPropagation();
                this.context.session.showMenu({
                    items: this.props.actions.map((a) => a.toMenuItem()),
                }, e.currentTarget as HTMLElement);
            };

            const classes = classNames({
                'fa': true,
                ['fa-ellipsis-h']: true
            });

            actionFragments.push((
                <li key="more" onClick={showMenu}>
                    <i className={classes} />
                </li>
            ));
        }

        return <ul className="actions">{actionFragments}</ul>;
    }

    private handleActionClick(action: HeaderAction) {
        if (action.callback) {
            action.callback();
        }
    }
}

export class HeaderAction {
    constructor(public label: string, public icon: string, public callback: () => void, public overflow: boolean = false) {
    }

    toMenuItem(): IMenuItem {
        return {
            label: this.label,
            icon: this.icon,
            onClick: () => this.callback()
        };
    }
}
