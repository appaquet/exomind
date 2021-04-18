import classNames from "classnames";
import React from "react";
import { EntityTrait, TraitIcon } from "../../store/entities";

interface IProps {
    trait?: EntityTrait<unknown>;
    icon?: TraitIcon;
}

export default class EntityIcon extends React.Component<IProps> {
    render(): React.ReactNode {
        if (!this.props.trait) {
            return this.renderTraitIcon(this.props.icon);
        }

        return this.renderTraitIcon(this.props.trait.icon);
    }

    private renderTraitIcon(icon: TraitIcon): React.ReactFragment {
        if ('fa' in icon) {
            return this.renderFaIcon(icon.fa);
        } else if ('emoji' in icon) {
            return this.renderEmoji(icon.emoji);
        }
    }

    private renderFaIcon(icon: string): React.ReactFragment {
        const iconClasses = classNames({
            [`fa-${icon}`]: true,
            fa: true,
            'entity-icon': true
        });
        const style = {
            padding: '2px'
        };

        return (
            <span className={iconClasses} style={style} />
        );
    }

    private renderEmoji(emoji: string): React.ReactFragment {
        const classes = classNames({
            'entity-icon': true,
            'emoji': true,
        });

        return <span className={classes}>{emoji}</span>;
    }

}