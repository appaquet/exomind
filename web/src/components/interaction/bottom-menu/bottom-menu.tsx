import React from 'react';
import { IAction, IActionResult } from '../../../utils/actions';
import { CancellableEvent } from '../../../utils/events';
import classNames from 'classnames';
import { Context as ShortcutContext, IMapping, ListenerToken, Shortcuts, } from '../../../shortcuts';
import _ from 'lodash';
import "./bottom-menu.less";

export type BottomMenuItem = IAction | 'divider';

interface IProps {
    items: BottomMenuItem[];
    shortcuts?: IActionShortcut[];
    onExecuted?: (action: IAction, result: IActionResult) => void;
}

export interface IActionShortcut {
    shortcutKey: string | string[];
    disabledContexts?: ShortcutContext[];
    actionKey: string;
}

export class BottomMenu extends React.Component<IProps> {
    private shortcutToken?: ListenerToken;

    constructor(props: IProps) {
        super(props);
        this.bindShortcut();
    }

    componentWillUnmount(): void {
        if (this.shortcutToken) {
            Shortcuts.unregister(this.shortcutToken);
        }
    }

    componentDidUpdate(): void {
        this.bindShortcut();
    }

    render(): React.ReactNode {
        return <div className="bottom-menu">
            <ul>
                {this.props.items.flatMap((action, i) => {
                    if (action === 'divider') {
                        return [<li className="divider" key={i}>&nbsp;</li>];
                    } else {
                        if (action.disabled) {
                            return [];
                        }

                        return [
                            <li className="action" key={action.label} onClick={(e) => this.handleExecuteAction(e, action)}>
                                <i className={classNames({
                                    'fa': true,
                                    ['fa-' + action.icon]: true,
                                })} />
                            </li>
                        ];
                    }
                })}
            </ul>
        </div>;
    }

    private bindShortcut() {
        if (this.shortcutToken) {
            Shortcuts.unregister(this.shortcutToken);
            this.shortcutToken = null;
        }

        if (!this.props.shortcuts) {
            return;
        }

        const indexedActions = _.keyBy(this.props.items, 'key');
        const connectedShortcuts: IMapping[] = this.props.shortcuts
            .map(shortcut => {
                const action = indexedActions[shortcut.actionKey] as IAction;
                if (!action) {
                    throw new Error(`Action with key ${shortcut.actionKey} not found`);
                }

                return {
                    key: shortcut.shortcutKey,
                    disabledContexts: shortcut.disabledContexts,
                    callback: (event: KeyboardEvent): boolean => {
                        this.handleExecuteAction(event, action);
                        return true;
                    }
                };
            });

        this.shortcutToken = Shortcuts.register(connectedShortcuts);
    }

    private async handleExecuteAction(e: CancellableEvent, action: IAction | null): Promise<boolean> {
        if (!action) {
            return false;
        }

        const res = await action.execute(e);
        this.props.onExecuted?.(action, res);

        return true;
    }
}

