$(function () {
    const placeholder = '<div class="dl-overlay"><div class="card">\n  <div class="progress">\n    <svg viewBox="0 0 36 36" stroke="#cccccc" stroke-width="2.5" fill="none">\n      <circle cx="18" cy="18" r="15.9155"/>\n      <circle cx="18" cy="18" r="15.9155" stroke="#4cc790"/>\n    </svg>\n    <div class="number">\n      <p><span>0</span>%</p>\n    </div>\n<div style="position: absolute;top: 65%;left: 33%;"><h2>加載中</h2></div> \n  </div>\n<div class="retry_img">\n      <span style="\n    cursor: pointer;\n    display: none;\n    font-size: 1rem;\n    border: 2px solid #ffffff;\n    border-radius: 5px;\n    color: white;\n    padding: 8px 10px;\n    background-color: #242424;\n    top: 17px;\n    position: relative;\n">加载失败,请点击重试</span>\n  </div>';
    
    const setProgress = ($element, percentage, message) => {
      $element.css('strokeDasharray', percentage + ' 100');
      $element.find('.number span').text(Math.round(percentage));
      if (message !== undefined) {
        $element.find('h2').html(message);
      }
    };
  
    webpMachine.webpSupport.then(webpSupported => {
      var isFirstAttempt = false;
      var prefix = 'my';
      let secretKey = '2ecret';
      let appendedKey = '782ec';
      let suffix = 'ret';
      prefix += secretKey + appendedKey + suffix;
      var maxHeight = 300; // example value, change it as needed
  
      $('img.lazy_img').each(function () {
        $(this).attr('data-original', '/static/images/imagecover.png');
        $(this).on('load', function () {
          if ($(this).attr('src').indexOf('blob:') > -1 || $(this).attr('src').indexOf('base64') > -1) {
            $(this).css('width', 'auto').css('display', 'flex').css('min-height', '0px');
            window.URL.revokeObjectURL($(this).attr('src'));
          }
        });
      });
  
      async function loadImages(url, $element, retryCount) {
        let imageHosts = $('.img-hosts').attr('data-img-hosts');
        if (imageHosts) {
          imageHosts = window.atob(imageHosts);
          imageHosts = imageHosts.split(',');
          for (let i = 0; i < imageHosts.length; i++) {
            if (i >= retryCount) {
              let urlObj = new URL(url);
              url = url.replace(urlObj.hostname, imageHosts[i]);
              break;
            }
          }
          setProgress($($element).prev(), 0, 'Loading');
          await downloadImage(url, $element, retryCount);
        }
      }
  
      try {
        lazyLoadImages();
      } catch (error) {}
  
      function lazyLoadImages() {
        $('img.lazy_img').lazyload({
          threshold: 1500,
          effect: 'fadeIn',
          load: function () {
            $(placeholder).insertBefore($(this));
            if ($(this).outerHeight() > maxHeight) {
              maxHeight = $(this).outerHeight();
            }
            $(this).prev().css('height', maxHeight);
            let imgElement = $(this)[0];
            let srcAttribute = imgElement.getAttribute('src');
            if (srcAttribute.indexOf('blob:') < 0 && srcAttribute.indexOf('base64') && srcAttribute.indexOf('/book/content/') < 0) {
              downloadImage(imgElement.getAttribute('data-r-src'), imgElement);
            }
          }
        });
      }
  
      async function downloadImage(url, $element, retryCount = 0) {
        let xhr = new XMLHttpRequest();
        xhr.open('GET', url, true);
        xhr.addEventListener('progress', function (event) {
          let percentage = event.loaded / (event.loaded + 6000) * 100;
          setProgress($($element).prev(), percentage, 'Loading');
        });
        xhr.responseType = 'arraybuffer';
        xhr.onerror = async function () {
          $($element).prev().css('height', '500px');
          let imageHosts = $('.img-hosts').attr('data-img-hosts');
          imageHosts = window.atob(imageHosts);
          imageHosts = imageHosts.split(',');
          if (retryCount < imageHosts.length) {
            retryCount += 1;
            $($element).prev().find('.number').html('<span>Retrying ' + retryCount + '</span>');
            $($element).prev().find('h2').html('');
            await loadImages(url, $element, retryCount);
          } else if ($($element).prev().hasClass('dl-overlay')) {
            $($element).prev().addClass('error-img');
            $($element).prev().css('strokeDasharray', '100 100');
            $($element).prev().find('.number').html('<span>Failed</span>');
            $($element).prev().find('h2').html('');
            if ($($element).prev().find('.retry_img > span').css('display') == 'none') {
              $($element).prev().find('.retry_img > span').show();
              $($element).prev().find('.retry_img > span').on('click', function () {
                console.log(retryCount);
                $($element).prev().removeClass('error-img');
                loadImages(url, $element, retryCount + 1);
              });
            }
          }
        };
        xhr.onload = async function () {
          if (xhr.readyState == 4) {
            if (xhr.status == 200) {
              $element.src = await decryptImage(xhr.response, prefix, $element);
              if ($($element).prev().hasClass('dl-overlay')) {
                $($element).prev().remove();
              }
            } else if (retryCount == 0) {
              retryCount += 1;
              await downloadImage(url, $element, retryCount);
            }
          }
        };
        xhr.send();
      }
  
      function bytesToUint8Array(bytes) {
        const length = bytes.sigBytes;
        const words = bytes.words;
        const uint8Array = new Uint8Array(length);
        var index = 0;
        var wordIndex = 0;
        while (true) {
          if (index == length) break;
          var word = words[wordIndex++];
          uint8Array[index++] = (word & 0xff000000) >>> 24;
          if (index == length) break;
          uint8Array[index++] = (word & 0xff0000) >>> 16;
          if (index == length) break;
          uint8Array[index++] = (word & 0xff00) >>> 8;
          if (index == length) break;
          uint8Array[index++] = word & 0xff;
        }
        return uint8Array;
      }
  
      const delay = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));
  
      const readDataURL = file => {
        let fileReader = new FileReader();
        fileReader.readAsDataURL(file);
        return new Promise(resolve => {
          fileReader.onloadend = () => {
            resolve(fileReader.result);
          };
        });
      };
  
      async function decryptImage(data, key, imageElement) {
        let encryptedData = data;
        let parsedKey = CryptoJS.enc.Utf8.parse(key);
        let wordArray = CryptoJS.lib.WordArray.create(encryptedData);
        let decryptedData = CryptoJS.AES.decrypt({
            ciphertext: wordArray
        }, parsedKey, {
            iv: parsedKey,
            padding: CryptoJS.pad.Pkcs7
        });
        let decryptedBytes = bytesToUint8Array(decryptedData);
        let decryptedDataURL = '';
        if (!webpSupported) {
            let retryCount = 0;
            while (decryptedDataURL === '' && retryCount < 100) {
                retryCount++;
                if (!isFirstAttempt) {
                    isFirstAttempt = true;
                    let webpData = await webpMachine.decode(decryptedBytes);
                    decryptedDataURL = webpData;
                    isFirstAttempt = false;
                }
                decryptedDataURL === '' && await delay(300);
            }
            decryptedDataURL == '' && console.log($(imageElement).attr('data-sort'), 'fail!!');
        } else {
            let blob = new Blob([decryptedBytes]);
            decryptedDataURL = URL.createObjectURL(blob);
        }
        return decryptedDataURL;
     }

    });
});
