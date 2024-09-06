import { Instance, createPopper, Modifier, VirtualElement, Placement } from "@popperjs/core";
import classNames from "classnames";
import React, { MouseEvent as ReactMouseEvent } from "react";
import { ListenerToken, Shortcuts } from "../../shortcuts";
import './menu.less';

export interface IMenu {
    items: IMenuItem[];
    reference?: HTMLElement;
    mouseEvent?: React.MouseEvent;
    placement?: Placement;
}

export interface IMenuItem {
    label: string;
    icon?: string;
    onClick: (e: ReactMouseEvent) => void;
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
        this.createPopper();
        document.addEventListener('click', this.handleClick);
        document.addEventListener('wheel', this.handleClick);
        Shortcuts.activateContext('contextual-menu');
    }

    componentWillUnmount(): void {
        this.popper?.destroy();
        document.removeEventListener('click', this.handleClick);
        document.removeEventListener('wheel', this.handleClick);
        Shortcuts.unregister(this.shortcutToken);
        Shortcuts.deactivateContext('contextual-menu');
    }

    componentDidUpdate(prevProps: Readonly<IProps>): void {
        if (this.props.menu.reference != prevProps.menu.reference || this.props.menu.mouseEvent != prevProps.menu.mouseEvent) {
            this.createPopper();
        }
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
                        } else {
                            iconClass = classNames({
                                icon: true,
                            });
                        }

                        const onClick = (e: ReactMouseEvent) => {
                            item.onClick(e);
                            this.props.onClose?.();
                        };

                        return (
                            <li key={index} onClick={onClick}>
                                <span className={iconClass} />
                                <span className="item-label">{item.label}</span>
                            </li>
                        );
                    })}
                </ul>
            </div>
        );
    }

    private createPopper() {
        this.popper?.destroy();

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

        let placement: Placement;
        let reference: Element | VirtualElement;
        if (this.props.menu.mouseEvent) {
            const event = this.props.menu.mouseEvent;
            reference = {
                getBoundingClientRect(): DOMRect {
                    return {
                        top: event.clientY,
                        bottom: event.clientY,
                        left: event.clientX,
                        right: event.clientX,
                        width: 0,
                        height: 0,
                    } as unknown as DOMRect;
                },
                contextElement: this.props.menu.reference || document.body,
            };

            placement = this.props.menu.placement || 'right-start';
        } else if (this.props.menu.reference) {
            reference = this.props.menu.reference;
            placement = this.props.menu.placement ?? 'left-start';
        } else {
            throw new Error('no reference element or mouse event provided');
        }

        this.popper = createPopper(reference, this.menuDiv.current, {
            placement: placement,
            modifiers: [hideWatcher],
        });
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
    };
}