import classNames from 'classnames';
import { observer } from 'mobx-react';
import React from 'react';
import { SelectedItem, Selection } from '../../objects/entity-list/selection';
import Column from './column';
import { ColumnConfig, ColumnConfigs } from './columns-config';
import { ListenerToken, Shortcuts } from '../../../shortcuts';
import './columns.less';

interface IProps {
    config: string;
    onConfigChange: (config: ColumnConfigs) => void;
}

interface IState {
    activeColumn: number;
    config: ColumnConfigs;
}

@observer
export default class Columns extends React.Component<IProps, IState> {
    private shortcutToken: ListenerToken;

    constructor(props: IProps) {
        super(props);

        this.state = {
            activeColumn: 0,
            config: this.makeConfig(props),
        };

        this.shortcutToken = Shortcuts.register([
            {
                key: 'Ctrl-b ArrowRight',
                callback: this.handleShortcutNext,
                disabledContexts: ['modal'],
            },
            {
                key: 'ArrowRight',
                callback: this.handleShortcutNext,
                disabledContexts: ['input', 'text-editor', 'modal'],
            },
            {
                key: 'Ctrl-b ArrowLeft',
                callback: this.handleShortcutPrev,
                disabledContexts: ['modal'],
            },
            {
                key: 'ArrowLeft',
                callback: this.handleShortcutPrev,
                disabledContexts: ['input', 'text-editor', 'modal'],
            },
            {
                key: 'Ctrl-b x',
                callback: this.handleShortcutClose,
            }
        ]);
    }

    componentWillUnmount(): void {
        Shortcuts.unregister(this.shortcutToken);
    }

    componentDidUpdate(): void {
        const prevConfig = this.state.config;
        const newConfig = this.makeConfig(this.props);

        if (!prevConfig.equals(newConfig)) {
            let activeColumn = this.state.activeColumn;

            if (this.state.activeColumn >= this.state.config.parts.length) {
                // we can't go beyond the last column
                activeColumn = this.state.config.parts.length - 1;
            }

            this.setState({
                config: newConfig,
                activeColumn: activeColumn
            });
        }
    }

    render(): React.ReactNode {
        const renderedColumns = this.renderColumns();
        const nbColumns = renderedColumns.length;
        const classes = `columns count-${nbColumns}`;
        return (
            <div className={classes}>
                {renderedColumns}
            </div>
        );
    }

    private renderColumns() {
        return this.state.config.parts.flatMap((columnConfig, colId) => {
            if (columnConfig.isMultiple) {
                return [];
            }

            let selectionItems: SelectedItem[] = [];
            const nextColumnConfig = this.state.config.parts[colId + 1];
            if (nextColumnConfig) {
                if (nextColumnConfig.isMultiple) {
                    selectionItems = Array.from(nextColumnConfig.values.map((col) => this.configToSelection(col)));
                } else {
                    selectionItems = [this.configToSelection(nextColumnConfig)];
                }
            }

            const selection = new Selection(Array.from(selectionItems.filter((col => !!col))));
            if (nextColumnConfig?.isMultiple ?? false) {
                selection.withForceMulti();
            }

            const colClass = `column-container-${colId}`;
            const classes = classNames({
                'column-container': true,
                [colClass]: true
            });
            const colKey = this.columnKey(columnConfig);
            const active = this.state.activeColumn == colId;
            const canClose = this.canCloseColumn(this.state.config, colId);

            return [(
                <div
                    className={classes}
                    key={colKey}
                    onMouseEnter={() => this.onColumnHovered(colId)}
                    onMouseOver={() => this.onColumnHovered(colId)}>

                    <Column
                        key={colKey}
                        columnId={colId}
                        columnConfig={columnConfig}
                        active={active}
                        onClose={canClose ? () => this.handleColumnClose(colId) : undefined}
                        selection={selection}
                        onSelectionChange={(selection) => this.handleColumnItemSelect(colId, selection)}
                    />
                </div>
            )];
        });
    }

    private handleShortcutNext = () => {
        let activeColumn = this.state.activeColumn;
        activeColumn++;

        if (activeColumn >= this.state.config.parts.length) {
            activeColumn = this.state.config.parts.length - 1;
        }

        this.setState({ activeColumn });

        return true;
    };

    private handleShortcutPrev = () => {
        let activeColumn = this.state.activeColumn;
        activeColumn--;

        if (activeColumn < 0) {
            activeColumn = 0;
        }

        this.setState({ activeColumn });

        return true;
    };

    private handleShortcutClose = () => {
        this.handleColumnClose(this.state.activeColumn);
        return true;
    };

    private onColumnHovered(columnId: number): void {
        if (Shortcuts.usedRecently || this.state.activeColumn == columnId) {
            return;
        }

        this.setState({
            activeColumn: columnId
        });
    }

    private makeConfig(props: IProps): ColumnConfigs {
        props = props || this.props;
        let config = ColumnConfigs.fromString(props.config);
        if (config.empty) {
            config = ColumnConfigs.forInbox();
        }
        return config;
    }

    private columnKey(config: ColumnConfig): string {
        if (config.isSearch) {
            // prevent recreating component for every keystroke
            return 'search';
        } else {
            return config.toString();
        }
    }

    private configToSelection(config: ColumnConfig): SelectedItem {
        if (config.isEntity) {
            return SelectedItem.fromEntityId(config.first);
        } else if (config.isTrait) {
            return SelectedItem.fromEntityTraitId(config.first, config.second);
        }
    }

    private handleColumnItemSelect(colId: number, selection: Selection) {
        let columnsConfig = this.state.config;
        if (selection && !selection.isEmpty) {
            const selectionConfigs = Array.from(selection.items.map((item) => {
                if (item.traitId) {
                    return ColumnConfig.forTrait(item.entityId, item.traitId);
                } else {
                    return ColumnConfig.forEntity(item.entityId);
                }
            }));

            let selectionConfig: ColumnConfig;
            if (selection.isMulti) {
                selectionConfig = ColumnConfig.forMultiple(selectionConfigs);
            } else {
                selectionConfig = selectionConfigs[0];
            }

            if (selectionConfig) {
                columnsConfig = columnsConfig.set(colId + 1, selectionConfig);
            }
        } else {
            columnsConfig = columnsConfig.unset(colId + 1);
        }

        this.props.onConfigChange(columnsConfig);
    }

    private handleColumnClose(colId: number) {
        // we allow closing any columns, as long as it is not the last one
        if (!this.canCloseColumn(this.state.config, colId)) {
            return;
        }

        if (this.state.activeColumn == colId) {
            this.setState({
                activeColumn: Math.max(this.state.activeColumn - 1, 0),
            });
        }

        const columnsConfig = this.state.config.pop(colId);
        this.props.onConfigChange(columnsConfig);
    }

    private canCloseColumn(config: ColumnConfigs, colId: number): boolean {
        return colId > 0 || config.parts.length > 1;
    }
}

