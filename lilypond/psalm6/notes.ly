melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 e2 e4 e4 d2 d2 e4( g2 fis4) g2 r2 \break

  % Line 2
  b2 b4 d4 c4 b4 a2 g2 r2 \break

  % Line 3
  d2 e4 g4 fis4 e4 b'2 r2 \break

  % Line 4
  d2 d4 c4 b4 a4 g2 e2 r2 \break

  % Line 5
  a2 a4 g4 fis4 e4 g2 d2 r2 \break

  % Line 6
  e2 b'4 b4 a2 fis2 e1 \bar "|."
}
