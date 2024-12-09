#!/bin/bash

# GitHub 저장소 정보
REPO="furiosa-ai/mlperf-log-parser"
BINARY_NAME="mlperf-log-parser"
PACKAGE_NAME=""

OS_TYPE=$(uname -s)

case $OS_TYPE in
    Linux)
        # 리눅스 배포판 감지
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            case $ID in
                alpine)
                    PACKAGE_NAME="mlperf-log-parser-alpine-x86_64.tar.gz"
                    ;;
                centos)
                    PACKAGE_NAME="mlperf-log-parser-centos-x86_64.rpm"
                    ;;
                ubuntu)
                    PACKAGE_NAME="mlperf-log-parser-ubuntu-x86_64.tar.gz"
                    ;;
                *)
                    echo "지원하지 않는 리눅스 배포판입니다"
                    exit 1
                    ;;
            esac
        else
            echo "알 수 없는 리눅스 배포판입니다"
            exit 1
        fi
        ;;
    Darwin)
        PACKAGE_NAME="mlperf-log-parser-macos-x86_64.tar.gz"
        ;;
    *)
        echo "지원하지 않는 운영체제입니다"
        exit 1
        ;;
esac

# 최신 릴리스 URL 가져오기
LATEST_RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
DOWNLOAD_URL=$(curl -s $LATEST_RELEASE_URL | grep "browser_download_url.*${PACKAGE_NAME}" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "오류: 감지된 OS에 대한 최신 릴리스 다운로드 URL을 찾을 수 없습니다"
    exit 1
fi

# 임시 디렉토리 생성
TMP_DIR=$(mktemp -d)
echo "임시 디렉토리 사용 중: $TMP_DIR"
cd $TMP_DIR

# 패키지 다운로드
echo "최신 릴리스 다운로드 중: $DOWNLOAD_URL"
curl -L -o $PACKAGE_NAME $DOWNLOAD_URL

# 패키지 설치
if [[ "$PACKAGE_NAME" == *".tar.gz" ]]; then
    tar -zxf $PACKAGE_NAME
    BINARY_PATH=$(find . -type f -name "$BINARY_NAME")

    echo "$BINARY_PATH 설치 중"
    chmod +x $BINARY_PATH
    sudo mv $BINARY_PATH /usr/local/bin/
elif [[ "$PACKAGE_NAME" == *".rpm" ]]; then
    sudo rpm -i $PACKAGE_NAME
fi

# 임시 디렉토리 정리
cd - > /dev/null
rm -rf $TMP_DIR

echo "설치가 완료되었습니다! 이제 'mlperf-log-parser' 명령어를 사용할 수 있습니다."