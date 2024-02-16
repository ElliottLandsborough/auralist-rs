import React from 'react';

export const LoadingSvg = () => {
  return (
    <svg class="loading-spinner" width="200" height="200" display="block" preserveAspectRatio="xMidYMid" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <circle cx="50" cy="50" r="32" fill="none" stroke="#0057b7" stroke-dasharray="50.26548245743669 50.26548245743669" stroke-linecap="round" stroke-width="8">
        <animateTransform attributeName="transform" dur="1.6949152542372883s" keyTimes="0;1" repeatCount="indefinite" type="rotate" values="0 50 50;360 50 50"/>
      </circle>
      <circle cx="50" cy="50" r="23" fill="none" stroke="#fd0" stroke-dasharray="36.12831551628262 36.12831551628262" stroke-dashoffset="36.1" stroke-linecap="round" stroke-width="8">
        <animateTransform attributeName="transform" dur="1.6949152542372883s" keyTimes="0;1" repeatCount="indefinite" type="rotate" values="0 50 50;-360 50 50"/>
      </circle>
    </svg>
  )
}