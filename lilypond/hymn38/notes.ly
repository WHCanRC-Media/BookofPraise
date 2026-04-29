melody = \relative c' {
  \clef treble
  \key a \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'4 e4 fis4 fis4 e4 d4 cis8( b8) a4 \break

  % Line 2
  a'4 gis8( fis8) b4 a8( gis8) fis4 fis4 e2 \break

  % Line 3
  a4 b4 cis4 b8( cis8) d8( cis8) b8( a8) b4 e,4 \break

  % Line 4
  a4 b4 cis4 b8( cis8) d8( cis8) b8( a8) b2 \break

  % Line 5
  a4 a8( g8) fis4 fis4 b4 b8( a8) gis2 \break

  % Line 6
  a4 b4 cis4 d8( cis8) b4 b4 a2 \bar "|."
}
