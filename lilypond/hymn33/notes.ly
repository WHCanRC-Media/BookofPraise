melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r4 fis4 fis4 g2 fis4 a4 a4 b2 a4 a4 d4 cis2 d2 \break

  % Line 2
  a4 a4 a4 b2 a4 a4( g4) fis4 a2 \break

  % Line 3
  fis4 fis4 fis4 fis2 fis4 fis4( e4) d4 e2 \break

  % Line 4
  a4 a4 a4 b2 a4 a4( g4) fis4 a2 r4 a4 d4 cis2 d2 \bar "|."
}
