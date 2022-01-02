import { Instance, createPopper, Modifier } from "@popperjs/core";
import classNames from "classnames";
import React from "react";
import { ListenerToken, Shortcuts } from "../../shortcuts";
import './menu.less';

export interface IMenu {
    reference?: HTMLElement;
    items: IMenuItem[];
}

export interface IMenuItem {
    label: string;
    icon?: string;
}

interface IProps {
    menu: IMenu;
    onClose?: () => void;
}

export class ContextualMenu extends React.Component<IProps> {
    private menuDiv: React.RefObject<HTMLDivElement> = React.createRef();
    private popper?: Instance;
    private shortcutToken: ListenerToken;

    constructor(props: IProps) {
        super(props);

        this.shortcutToken = Shortcuts.register([
            {
                key: 'Escape',
                callback: () => {
                    console.log('pressed');
                    if (this.props.onClose) {
                        this.props.onClose();
                        return true;
                    } else {
                        return false;
                    }
                },
            }
        ]);
    }

    componentDidMount(): void {
        // monitors if menu is hidden by popper because reference elements has disappear from view
        const hideWatcher: Modifier<string, unknown> = {
            name: 'hideWatcher',
            enabled: true,
            phase: 'main',
            fn: () => {
                const attr = this.menuDiv.current?.attributes.getNamedItem('data-popper-reference-hidden');
                if (attr) {
                    this.props.onClose?.();
                }
            },
        };

        this.popper = createPopper(this.props.menu.reference, this.menuDiv.current, {
            placement: 'left-start',
            modifiers: [hideWatcher],
        });
        document.addEventListener('click', this.handleClick);
        Shortcuts.activateContext('contextual-menu');
    }

    componentWillUnmount(): void {
        this.popper?.destroy();
        document.removeEventListener('click', this.handleClick);
        Shortcuts.unregister(this.shortcutToken);
        Shortcuts.deactivateContext('contextual-menu');
    }

    render(): React.ReactNode {
        return (
            <div className="contextual-menu" ref={this.menuDiv}>
                <ul>
                    {this.props.menu.items.map((item, index) => {
                        let iconClass;
                        if (item.icon) {
                            iconClass = classNames({
                                icon: true,
                                fa: true,
                                ['fa-' + item.icon]: true,
                            });
                        }

                        return (
                            <li key={index}>
                                {item.icon && <span className={iconClass} />}
                                <span className="item-label">{item.label}</span>
                            </li>
                        );
                    })}
                </ul>
            </div>
        );
    }

    private handleClick = (e: MouseEvent): void => {
        // check if click is outside of menu
        let found = false;
        let target = e.target as HTMLElement;
        for (let i = 0; i < 10; i++) {
            if (!target) {
                break;
            } else if (target === this.menuDiv.current) {
                found = true;
                break;
            } else {
                target = target.parentElement;
            }
        }

        if (found) {
            // nothing to do, we clicked on the menu
            return;
        } else {
            this.props.onClose?.();
        }
    }
}