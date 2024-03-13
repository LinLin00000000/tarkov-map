import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import styled from 'styled-components';

// Styled components
const SettingsWrapper = styled.div`
    padding: 20px;
    background: #f7f7f7;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
`;

const StyledForm = styled.form`
    display: flex;
    flex-direction: column;
    gap: 10px;
`;

const StyledLabel = styled.label`
    display: block;
    font-weight: bold;
    margin-bottom: 5px;
`;

const StyledInput = styled.input`
    padding: 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
    margin-bottom: 10px;
`;

const StyledCheckboxLabel = styled.label`
    display: flex;
    align-items: center;
`;

const StyledCheckbox = styled.input.attrs({ type: 'checkbox' })`
    margin-right: 10px;
`;

const SubmitButton = styled.button`
    padding: 10px 15px;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
`;

const Settings = ({ closeModal }) => {
    const [autoDeleteScreenshot, setAutoDeleteScreenshot] = useState(true);
    const [autoScreenshotByKeyWRelease, setAutoScreenshotByKeyWRelease] = useState(false);
    const [autoMaximizeMinimize, setAutoMaximizeMinimize] = useState(false);
    const [autoAppearedByShortcutKey, setAutoAppearedByShortcutKey] = useState(false);
    const [shortcutKey, setShortcutKey] = useState('');

    // 加载时获取设置
    useEffect(() => {
        const savedSettings = localStorage.getItem('settings');
        if (savedSettings) {
            const settings = JSON.parse(savedSettings);
            setAutoDeleteScreenshot(settings.auto_delete_screenshot);
            setAutoScreenshotByKeyWRelease(settings.auto_screenshot_by_key_w_release);
            setAutoMaximizeMinimize(settings.auto_maximize_minimize);
            setAutoAppearedByShortcutKey(settings.auto_appeared_by_shortcut_key);
            setShortcutKey(settings.shortcut_key);
        }
    }, []);

    // 保存设置
    const saveSettings = async () => {
        const settings = {
            auto_delete_screenshot: autoDeleteScreenshot,
            auto_screenshot_by_key_w_release: autoScreenshotByKeyWRelease,
            auto_maximize_minimize: autoMaximizeMinimize,
            auto_appeared_by_shortcut_key: autoAppearedByShortcutKey,
            shortcut_key: shortcutKey,
        };

        localStorage.setItem('settings', JSON.stringify(settings));

        invoke('save_settings', { settings: JSON.stringify(settings) })
            .then(() => console.log('Settings saved successfully'))
            .catch((error) => console.error('Error saving settings:', error));
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        closeModal();
        await saveSettings();
    };

    return (
        <SettingsWrapper>
            <StyledForm onSubmit={handleSubmit}>
                <StyledCheckboxLabel>
                    <StyledCheckbox
                        checked={autoDeleteScreenshot}
                        onChange={(e) => setAutoDeleteScreenshot(e.target.checked)}
                    />
                    自动删除截图
                </StyledCheckboxLabel>
                <StyledCheckboxLabel>
                    <StyledCheckbox
                        checked={autoScreenshotByKeyWRelease}
                        onChange={(e) => setAutoScreenshotByKeyWRelease(e.target.checked)}
                    />
                    W键松开自动截图
                </StyledCheckboxLabel>
                <StyledCheckboxLabel>
                    <StyledCheckbox
                        checked={autoMaximizeMinimize}
                        onChange={(e) => setAutoMaximizeMinimize(e.target.checked)}
                    />
                    根据截图出现自动最小化
                </StyledCheckboxLabel>
                <StyledCheckboxLabel>
                    <StyledCheckbox
                        checked={autoAppearedByShortcutKey}
                        onChange={(e) => setAutoAppearedByShortcutKey(e.target.checked)}
                    />
                    使用截图快捷键自动最小化
                </StyledCheckboxLabel>
                <StyledLabel htmlFor="shortcut-key">游戏设置的截图快捷键:</StyledLabel>
                {/* See https://docs.rs/rdev/latest/rdev/enum.Key.html */}
                <span>(Tip: https://docs.rs/rdev/latest/rdev/enum.Key.html)</span>
                <StyledInput
                    type="text"
                    id="shortcut-key"
                    value={shortcutKey}
                    onChange={(e) => setShortcutKey(e.target.value)}
                />
                <SubmitButton type="submit">保存设置</SubmitButton>
            </StyledForm>
        </SettingsWrapper>
    );
};

export default Settings;
